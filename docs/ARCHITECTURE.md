# SoundGen — Mimari

> Durum: v0.1 — çekirdek motor + preset kütüphanesi + UI iskeleti
> Tarih: 24.10.2026

## 1. Ürün tezi

Oyun ses tasarımının %80'i tekrar eden iştir: 30 silah varyantı, 40 zombi homurtusu, 20 adım sesi.
SoundGen bu işi **parametrik, tohumlanabilir ve anında dinlenebilir** hale getirir.
Örnek (sample) kullanmaz: her ses, katmanlı bir sentez grafiğinden sıfırdan üretilir.

Üç sütun:

1. **Katman (Layer)** — her efekt 2-6 katmandan oluşur (transient / gövde / kuyruk / süs).
2. **Stil ekseni** — `style ∈ [0,1]`: 0 = gerçekçi/fiziksel, 1 = çizgi film/abartılı.
3. **Aile (Family)** — tek ses değil, aynı karakteri paylaşan tohumlu varyasyon kümesi.

## 2. Neden "offline render + oynat" (gerçek zamanlı grafik değil)

| Yaklaşım | Gecikme | Karmaşıklık | Parametre değişimi |
|---|---|---|---|
| Gerçek zamanlı DSP grafiği | ~5 ms | yüksek (kilit, tahsis yasağı, voice stealing) | örnek-hassas ama riskli |
| **Offline render + ring buffer** | render süresi + ~10 ms | düşük | her değişimde yeniden render |

1-2 saniyelik bir efekti Rust'ta render etmek **3-10 ms** sürüyor (ölçüldü: 3 preset / 0.03 s).
Bu yüzden v1'de gerçek zamanlı grafik yok: parametre değişince buffer yeniden üretilir,
cpal çıkışına verilir. Sonuç: tık sesi yok, kilit yok, deterministik, aynı kod hem UI hem toplu export için çalışır.

> Not: Araştırma raporu (03-stack.md) Windows'ta ≤10 ms için WASAPI exclusive/ASIO öneriyor.
> Bizim modelde algılanan gecikme render + tampon olduğu için paylaşımlı WASAPI yeterli.
> Gerekirse `sg-io::Player` arkasına WASAPI exclusive sürücü eklenir; API değişmez.

## 3. Kasa yerleşimi

```
crates/
  sg-dsp/        Rng (xoshiro256**), Noise (white/pink/brown/blue/velvet),
                 AdEnv/EnvGen, SVF (TPT) + RBJ biquad + Smoother,
                 Osc (PolyBLEP) + PitchEnv, ModalBank + inharmonic_modes,
                 Fx (DC, Saturator, Compressor, Freeverb, Limiter, widen)
  sg-engine/     Layer/LayerKind, Recipe/MasterChain, render(), AudioBuffer
                 (+ peak/rms/LUFS/trim/fade/normalize/DC analizi)
  sg-presets/    kategori başına bir modül: weapons, zombies, gore, foley,
                 loops, crates, celebration, ui, ambience
  sg-io/         WAV yazımı (16/24/32-bit, mono/stereo) + cpal Player
apps/
  soundgen/      egui 0.37 stüdyo arayüzü
  soundgen-cli/  toplu render / QA aracı
```

Bağımlılık yönü tek: `app -> presets -> engine -> dsp`. `sg-io` yalnızca engine'e bağlıdır.

## 4. Render hattı

```
Recipe
  ├─ Layer[0..n]  ── her katman kendi Rng alt-tohumuyla mono render
  │     delay → gain_db → style_multiplier(style) → equal-power pan
  ├─ MasterChain: DC → HP → low shelf → presence → drive → comp
  │                → reverb send → stereo width → limiter → peak normalize
  └─ AudioBuffer (interleaved stereo, 48 kHz)
```

- Her katman **kendi alt-tohumunu** alır: bir katmanı silmek diğerlerinin sesini değiştirmez.
- `render_length()` = en uzun katman kuyruğu + 50 ms güvenlik payı.
- Stil ekseni şimdilik: katman başına `style_gain[2]` interpolasyonu + yüksek stil değerlerinde hafif bitcrush.
  (Araştırma 12 sürücülü bir harita öneriyor — yol haritasında.)

## 5. Stil ekseni (gerçekçi ↔ çizgi film)

Araştırma bulgusu: stil, tek bir lineer interpolasyon değil; sürücü bazlı eğrilerle eşlenmeli.
v1'de uygulanan:

- katman kazancı (`style_gain`) — ör. gerçekçi kuyruk 0.7, çizgi film kuyruğu 1.6
- yüksek stilde hafif quantization (bitcrush) ve geniş stereo

v1.5'te eklenecek sürücüler (rapor 01, bölüm 3):
pitch envelope derinliği ×2.2-3.5, modal Q ×0.25, mistune artışı, spektral tilt,
drive artışı, kuyruk süresi ×1.5, wet artışı, width ×1.3.

## 6. Varyasyon stratejisi

- `Recipe::randomize(amount, seed)` — katman kazancı ±, pan ±0.08, gecikme ±15%,
  frekans ±%5-18 (log-normal), decay ±%22, modal detune ±30 cent.
- `render_variations(recipe, n, base_seed)` — aile üretimi.
- Aynı tohum = aynı ses (deterministik). Oyunda tekrar hissini kırmak için 8-12 varyasyon önerilir.

## 7. Çıktı sözleşmesi (v1: sadece WAV)

- 48 kHz, 16-bit (mobil) veya 24-bit, varsayılan stereo; UI kategorisine göre mono seçilebilir.
- `-1 dBFS` peak normalizasyon, 0.5 ms fade-in / 20 ms fade-out, sessizlik kırpma.
- İsimlendirme: `out/<kategori>/<recipe_id>.wav`, varyasyonlar `_v01`.
- (Rapor 04'teki JSON sidecar + LUFS hedefleri v1.5'e bırakıldı; kullanıcı v1 için "sadece WAV" dedi.)

## 8. Performans bütçesi

- Render: hedef < 20 ms/efekt (48 kHz, 2 s). Ölçüm: 3 preset toplam 30 ms.
- UI: yeniden render yalnızca değişimde, 40 ms debounce; waveform min/max decimation.
- Bellek: efekt başına ~1-2 MB (2 s stereo f32). 100 efekt ≈ 200 MB — sorun değil.

## 9. Yol haritası

| Sürüm | İçerik |
|---|---|
| v0.1 (bu) | DSP çekirdeği, ~55 preset, egui stüdyo, WAV export, varyasyon ailesi |
| v0.2 | Spektrogram + gelişmiş stil haritası, undo, preset kaydetme (RON), toplu export presetleri |
| v0.3 | Loop seam doğrulama, LUFS-M hedefleri, mono/stereo kategori politikası, JSON sidecar |
| v0.4 | Unity manifest + Addressables importer, FMOD/Wwise iskeleti |
| v1.0 | VST3 (nih-plug) veya runtime kütüphane, nöral doku katmanı (pluggable) |

## 10. Bilinen sınırlar

- Gerçek zamanlı parametre otomasyonu yok (her değişimde tam render).
- Granüler katman, gerçek örnek taraması değil: gürültü tanelerinden üretilir (doku için yeterli).
- Modal bank f32; 150 Hz altında uzun T60'larda hassasiyet kaybı olabilir (rapor 01 uyarısı).
- Loop'lar süre matematiğiyle hizalanır; otomatik seam doğrulaması v0.3'te.
