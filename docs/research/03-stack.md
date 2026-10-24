# 03 — Teknoloji Yığını Kararı (Stack Decision)

**Durum:** Karar önerisi (decision-grade)
**Kapsam:** SoundGen v1 — Windows 11 masaüstü, prosedürel SFX üretimi, anlık audition, canlı parametre düzenleme
**Tarih:** 24 Ekim 2026
**Yöntem:** Birincil kaynak (Microsoft Learn, resmi dokümanlar, crate/NuGet/npm registry) + üçüncü taraf ölçümler. Ölçüm ile tahmin ayrı etiketlendi: **[ÖLÇÜM]** / **[TAHMİN]** / **[TASARIM HEDEFİ]**.

---

## 0. Yönetici Özeti

Bu projenin teknoloji kararını belirleyen tek bir soru var: **"Tuşa bastığımda ses 10 ms içinde çıkacak mı, ve parametreyi çevirdiğimde tıkırtı olmadan anında değişecek mi?"** Bu soru, aday listesini ikiye bölüyor. Windows'ta WASAPI shared mode üzerinden konuşan *her* tarayıcı tabanlı çözüm (Web Audio/Electron/Tauri) ve Python, yapısal olarak bu eşiğin altına inemiyor; C++/JUCE, Rust ve C#/.NET ise `IAudioClient3` veya WASAPI exclusive ile rahatça iniyor.

İkinci belirleyici soru: **"Bu uygulamanın asıl işi DSP mi, yoksa UI mı?"** SoundGen bir synthesizer değil; bir *içerik üretim aracı*. İşin %60'ı waveform/spectrogram çizimi, knob'lar, katman ağacı, seed ailesi yönetimi, export pipeline'ı. DSP tarafı ise "bir saniyelik sesi 30-80 düğümden geçirip üretmek" — 48 kHz'de 128 örnekli blok başına 2,67 ms bütçenin çok altında kalan bir yük.

Bu iki gerçek birleşince öneri netleşiyor: **.NET 10 + Avalonia 12 (UI) + NAudio 3 (WASAPI/ASIO backend) + kendi yazdığımız SIMD DSP çekirdeği.** C++/JUCE teknik olarak eşit derecede iyi (4,11 vs 4,41 ağırlıklı skor) ama geliştiricinin bildiği diller arasında değil ve C++ bellek hataları bu projede en pahalı hata sınıfı. C# ayrıca Unity ile aynı dili konuştuğu için ileride "aynı DSP çekirdeğini Unity içinde çalıştırma" kapısını açık bırakıyor.

---

## 1. Üründen Türeyen Katı Gereksinimler

Aşağıdaki tablo, brifingdeki ürün gereksinimlerini ölçülebilir teknik gereksinimlere çeviriyor. Bu tablo, Bölüm 2'deki skor matrisinin ağırlıklarının da gerekçesi.

| # | Gereksinim | Ölçülebilir hedef | Neden bu hedef | Doğrulama yöntemi |
|---|---|---|---|---|
| R1 | **Round-trip / press-to-sound gecikmesi** | Tuş → ses ≤ **10 ms** (kabul edilebilir üst sınır 15 ms) | "Anlık audition" hissi; 20 ms üzeri enstrüman hissini bozar, 10 ms altı "kablosuz" hissi verir | Loopback RTL ölçümü (RTL Utility veya kendi probe'umuz), 100 tıklık dağılım |
| R2 | **Gerçek zamanlı parametre otomasyonu** | Parametre değişimi **tıklamasız**, ≤ 20 ms smoothing, örnek-hassas olay zamanlaması | Kullanıcı morph slider'ını sürüklerken zipper noise duymamalı | Spektral süreklilik testi + null test (parametre sabitken çıktı bit-eş) |
| R3 | **Per-sample DSP @ 48 kHz** | 128 örnekli blokta (2,67 ms) tam katman yığını; blok bütçesinin ≤ %50'si | 48 kHz, oyun standardı; 128 blok, 2,67 ms quantization | BenchmarkDotNet / criterion gate (CI'da fail eder) |
| R4 | **SIMD** | AVX2 (8×f32) yolu; skaler fallback | i5-11400H AVX2 + FMA var, **AVX-512 yok** (Tiger Lake-H consumer'da disable) | `Vector.IsHardwareAccelerated`, intrinsics testleri |
| R5 | **Çok iş parçacıklı offline batch** | 1.000 varyant × 2 s ≤ **30 s** (12 thread) | "Seed ailesi" üretimi toplu yapılır; kullanıcı beklememeli | Batch benchmark, `Parallel.For`/rayon ölçümü |
| R6 | **Zengin özel UI** | 10 s waveform + spectrogram @ 60 fps, 100+ knob, sürüklenebilir node'lar | Ürünün ana değeri görsel geri bildirim | Frame time profili (≤ 8 ms CPU), 4K'da 60 fps |
| R7 | **Paketleme + installer** | Per-user install, admin gerekmez, ≤ 150 MB, imzalı | Kurumsal olmayan tek geliştirici dağıtımı | Temiz VM'de kurulum testi |
| R8 | **Code signing** | EXE + installer imzalı, RFC3161 timestamp | SmartScreen ve AV false-positive'leri | `signtool verify /pa` |
| R9 | **Crash safety** | Ses thread'i kilit/alloc yapmaz; UI çökse bile ses kesilmez; proje autosave | Uzun oturumlar; kullanıcı kaybı kabul edilemez | Fault injection, 8 saatlik soak test |
| R10 | **Unity handoff** | 48 kHz WAV + JSON metadata + importer ayarları | Oyun tarafında sıfır sürtünme | Unity 6.3'te import testi |

**Kritik gözlem:** R1 ve R3 aynı şey değil. R1 cihaz/OS katmanı, R3 CPU katmanı. Bir yığın R3'te mükemmel olup R1'de başarısız olabilir (Web Audio tam olarak bu). Bu yüzden matriste latency ve DSP performansı ayrı kriterler.

---

## 2. Aday Yığınlar ve Skor Matrisi

### 2.1 Adaylar

- **A — C++ / JUCE 9** (`juce_audio_devices`, `juce_audio_processors`, `Component` tabanlı UI)
- **B — Rust** (`cpal` + `egui`/`eframe`; UI alternatifi `iced`)
- **C — C# / .NET 10** (`NAudio 3` + `Avalonia 12`, custom `SkiaSharp`/`DrawingContext` çizimi)
- **D — TypeScript + Web Audio AudioWorklet** (Electron 45 veya Tauri 2 kabuğu)
- **E — Python** (`numpy`/`scipy` offline + `sounddevice`/PortAudio canlı)

### 2.2 Kriterler ve ağırlıklar

Ağırlıklar ürün gereksinimlerinden türetildi; "skill fit" kriteri brifingde geliştiricinin C#, Python, TypeScript bildiği açıkça yazdığı için eklendi (aksi halde matris gerçekçi olmaz).

| Kriter | Ağırlık | Gerekçe |
|---|---|---|
| Latency / real-time I/O | 0,18 | R1 — ürünün varlık nedeni |
| DSP performansı (per-sample, SIMD) | 0,14 | R3, R4, R5 |
| UI yeteneği (custom drawing, layout, DPI) | 0,18 | R6 — işin en büyük kısmı |
| Geliştirme hızı | 0,13 | Tek geliştirici + AI agent |
| Ekosistem (DSP, dosya, codec, test) | 0,09 | Yeniden kullanım |
| Paketleme / installer | 0,05 | R7 |
| Test edilebilirlik | 0,05 | R9, R3 gate'leri |
| Geleceğe hazırlık (plugin, FMOD/Wwise, cross-platform) | 0,08 | v2 yolu |
| Ekip uyumu (mevcut yetkinlik) | 0,10 | Brifing: C#, Python, TS |
| **Toplam** | **1,00** | |

### 2.3 Matris (1–5, 5 = en iyi)

| Kriter (ağırlık) | A: C++/JUCE | B: Rust | C: C#/.NET | D: TS/Web | E: Python |
|---|---|---|---|---|---|
| Latency / RT I/O (0,18) | **5** | 4 | 4 | 2 | 1 |
| DSP performansı (0,14) | **5** | **5** | 4 | 2 | 2 |
| UI yeteneği (0,18) | **5** | 3,5 | **5** | 5 | 3 |
| Geliştirme hızı (0,13) | 2 | 3 | **5** | 5 | 5 |
| Ekosistem (0,09) | **5** | 4 | 4 | 2 | 3 |
| Paketleme (0,05) | 4 | **5** | 4 | 4 | 1 |
| Test edilebilirlik (0,05) | 2 | **5** | 4 | 3 | 4 |
| Geleceğe hazırlık (0,08) | **5** | 4 | 4 | 2 | 1 |
| Ekip uyumu (0,10) | 2 | 3 | **5** | 5 | 4 |
| **Ağırlıklı toplam** | **4,11** | **3,92** | **4,41** | **3,38** | **2,65** |

### 2.4 Skorların gerekçesi (kısa)

- **A / C++:** Latency'de kral. JUCE 2020'den beri `IAudioClient3` "Shared Low Latency" modunu, WASAPI exclusive'ı, ASIO'yu ve DirectSound'u tek API'de sunuyor; `AudioDeviceManager` ile 4 backend arasında geçiş bedava. UI tarafında `Graphics::drawImage`, `Path`, `OpenGLContext` ile 60 fps waveform çizmek kanıtlanmış bir yol. Ama C++ geliştirme hızı 2: derleme/link süreleri, header bağımlılığı, ve AI agent'ların C++'ta ürettiği bellek hataları bu projede en pahalı hata sınıfı. Test edilebilirlik 2: JUCE'ın test altyapısı yok, kendi harness'ını kurman gerekiyor.
- **B / Rust:** DSP ve test tarafında mükemmel (cargo test + `insta` snapshot + `criterion` gate'leri). Paketleme en iyisi (tek ~10 MB exe). Ama iki somut boşluk: (1) **`cpal` 0.18 Windows'ta WASAPI shared sunar; exclusive mode ve `IAudioClient3` düşük-gecikme yolu hazır gelmiyor** — bunun için `wasapi` (0.26) veya `windows` (0.62) crate'iyle kendi backend'ini yazman gerekir; (2) `egui` immediate-mode bir UI; 100+ knob, focus/erişilebilirlik, IME, pahalı text layout ve "pro audio tool" görünümü için her widget'ı elle yazarsın. Rust bilmiyorsan bu iki boşluk 2-3 haftalık ek süre demek.
- **C / C#:** Latency 4: **NAudio 3.3 `WasapiPlayer` doğrudan `IAudioClient3` low-latency shared mode, exclusive mode, MMCSS "Pro Audio" thread priority ve raw mode sunuyor** — yani R1 için eksik olan her şey hazır. `NAudio.Asio` paketi ASIO'yu da kapsıyor. DSP 4: .NET 10'da `System.Runtime.Intrinsics` (AVX2), `TensorPrimitives`, `Span<T>`, `NativeMemory` var; GC riski *disiplinle* sıfıra indirilebilir (steady-state'te sıfır allocation). UI 5: Avalonia 12 (Nisan 2026; güncel 12.1.4) XAML + Skia render; retained layout, data binding, styling, DPI, erişilebilirlik bedava; custom waveform için `DrawingContext`/`SKCanvas`/`WriteableBitmap` açık.
- **D / TS + Web Audio:** UI 5, dev speed 5 — ama latency 2. Chrome 153'te `renderSizeHint: 64` ile **loopback round-trip 25,69 ms** [ÖLÇÜM, macOS + RME Babyface], 128'de 28,58 ms. Windows'ta Chrome backend'i WASAPI *shared*; exclusive/ASIO yok. Yani R1 (≤10 ms) yapısal olarak imkânsız. Ayrıca DSP'yi WASM'a taşısan bile SIMD (wasm simd128 = 4×f32) ve GC (JS) maliyeti var. Bu aday **R1'i karşılamadığı için elenir**, UI hızı kurtarmaz.
- **E / Python:** Offline prototip için harika (numpy/scipy), ama real-time'da GIL + callback başına Python overhead + `sounddevice`'ın PortAudio buffering'i ile 15-40 ms aralığına düşer; paketleme (PyInstaller ~100-150 MB, AV false positive) ve UI (Qt lisansı veya Dear PyGui) zayıf. **Prototip aracı olarak değerli, ürün yığını değil.**

**Sonuç:** C# 4,41 ile birinci, C++ 4,11 ile ikinci, Rust 3,92 ile üçüncü. C++ ile C# arasındaki fark tek bir kriterden değil, "ekip uyumu + geliştirme hızı"nın "latency + DSP" farkını 0,3 puanla geçmesinden geliyor. C++ yetkinliği olsaydı tablo tersine dönerdi — bunu açıkça not ediyorum.

---

## 3. Windows'ta Gecikme: Gerçek Sayılar ve Nereden Geldikleri

### 3.1 Windows ses yığınının kendi gecikmesi

Microsoft'un kendi dokümanı [1] net: Windows 10 öncesi audio engine gecikmesi float için ~12 ms, buffer her zaman ~10 ms idi. Windows 10+ ile engine gecikmesi **1,3 ms**'e indi ve **buffer boyutu artık driver tarafından tanımlanıyor** — 10 ms, 5 ms, 3 ms, 1 ms olabilir.

İki mod var:

| Mod | Ne olur | Tipik period @48 kHz | Tipik render gecikmesi |
|---|---|---|---|
| **WASAPI shared** (varsayılan) | Audio engine karıştırır, APO'lar çalışır, resample olabilir | 10 ms (varsayılan); `IAudioClient3` ile **2,67–3 ms**'e kadar | 10–20 ms (default) / **3–6 ms** (IAudioClient3 min period) |
| **WASAPI exclusive** | Engine bypass, uygulama doğrudan driver buffer'ına yazar, cihaz tek uygulamaya kilitlenir | 1–3 ms (128 frame ≈ 2,67 ms) | **3–6 ms** |
| **ASIO** | Üretici driver'ı, en düşük ve en tutarlı | 64–256 frame | **2–6 ms** |

Üçüncü taraf ölçümler bu aralıkları doğruluyor: SoundBridge'in Ekim 2026 tarihli karşılaştırması [4] "ASIO 5–15 ms round-trip, WASAPI Exclusive ideal koşulda buna yaklaşır, WASAPI Shared 20–35 ms ekler" diyor; Stack Overflow'ta eski donanımda ASIO ~15 ms / WASAPI ~50 ms ölçülmüş. JUCE forumunda `IAudioClient3`'ün **48 kHz'de 2,67 ms**'e kadar izin verdiği belirtiliyor [5].

**Önemli ayrım:** Bunlar *render* gecikmesi. Kullanıcının algıladığı "tuşa bastım, ses çıktı" süresi = input event gecikmesi (1–5 ms) + uygulama kuyruğu (0–2,67 ms) + render gecikmesi + DAC. Yani **render 5 ms ise press-to-sound ~8–12 ms** olur. 10 ms hedefi için render'ı 5 ms'in altında tutmak şart.

### 3.2 Yığın bazında ulaşılabilir gecikme

| Yığın | Varsayılan yol | En iyi yol | Press-to-sound [TAHMİN] | Not |
|---|---|---|---|---|
| **C++ / JUCE** | `IAudioClient3` shared low-latency | WASAPI exclusive 64–128 frame / ASIO 64 | **6–11 ms** | Dört backend de hazır; JUCE 2020'den beri `IAudioClient3` |
| **Rust / cpal** | WASAPI shared, 10 ms period | cpal `asio` feature, veya `wasapi` crate'iyle kendi exclusive backend'in | **10–18 ms** (shared) / **6–11 ms** (ASIO) | Exclusive + IAudioClient3 hazır değil → ek iş |
| **C# / NAudio 3** | `WasapiPlayer` + `WithLowLatency()` (IAudioClient3) | `WithExclusiveMode()` + 128 frame; `NAudio.Asio` | **7–12 ms** | MMCSS, raw mode, default-device routing hazır |
| **TS / Web Audio** | Chrome WASAPI shared, 128 frame quantum | `renderSizeHint: 64` (Chrome 153+) | **25–60 ms** | Exclusive/ASIO yok; R1 karşılanmaz |
| **Python** | PortAudio WASAPI shared | `sd.WasapiSettings(exclusive=True)`, blocksize 128–256 | **15–40 ms** | Callback başına Python overhead + jitter |

Web Audio tarafındaki en somut veri: Jeff Kaufman'ın 2020 loopback ölçümünde Chrome **19–41 ms** (tutarsız), Firefox 14 ms, Reaper 11 ms [11]; 2026'da Classmethod, Chrome 153 + `renderSizeHint` ile macOS'te 64 frame'de **25,69 ms**, 128'de 28,58 ms, 256'da 31,48 ms loopback RTL ölçtü [12]. `AudioContext.baseLatency` ve `outputLatency` bu değerlerin sadece bir kısmını raporlar [13]. Windows'ta Chrome WASAPI shared kullandığı için bu sayıların üzerine pay eklemek gerekir. **Karar: D elenir.**

### 3.3 Buffering stratejisi (tüm yığınlar için aynı mantık)

1. **Device period:** 128 frame (2,67 ms) hedef; driver desteklemiyorsa 240 (5 ms) → 480 (10 ms) → 960 (20 ms) fallback merdiveni.
2. **Internal block:** Her zaman **128 frame**. Device period'dan bağımsız; iki thread arasında ring buffer.
3. **Safety blocks:** 2 blok (5,3 ms) — underrun'a karşı. Toplam queue = 3 blok = 8 ms.
4. **Pre-roll / clip cache:** Audition iki modda çalışır: (a) *düzenlenen* ses canlı graph'tan akar — parametre değişimi anında duyulsun diye; (b) yeni üretilen varyantlar arka planda render edilip **clip cache**'e alınır; A/B karşılaştırması, hızlı ardışık tetikleme ve export önizlemesi bu cache'den çalınır. Her iki modda da başlangıç 1 blok sınırına hizalanır (≤ 2,67 ms quantization), böylece transport jitter'ı sesi etkilemez.
5. **Ölçüm:** `IAudioClient::GetStreamLatency` + `IAudioClock::GetPosition` ile gerçek render gecikmesi; buna kendi komut-kuyruğu gecikmemizi ekleyip UI'da "Latency: 8,3 ms" olarak göster. Ölçmediğimiz sayıyı vaat etmiyoruz.

Bu strateji ile C# yığınında **6–12 ms press-to-sound** gerçekçi; "10 ms altı" hedefi tutturulabilir ama garanti değil — cihaza bağlı. UI'da cihaz bazlı bir latency göstergesi ve "Exclusive mode" toggle'ı bu yüzden zorunlu.

---

## 4. Yığından Bağımsız Ses Motoru Mimarisi

Bu bölüm hangi dil seçilirse seçilsin geçerli. Aynı graph hem real-time audition hem offline bounce için kullanılır — **tek kod yolu** ilkesi. Bu, "audition'da duyduğun ses export'ta çıkar" garantisinin tek yolu.

### 4.1 Graph modeli

```
Voice (bir SFX örneği)
└── Layer[] (gövde, transient, gore, tail, ambience…)
    └── Node graph (DAG, feedback için 1-sample delay izinli)
        ├── Source      : Noise(beyaz/pembe/brown), Wavetable, SamplePlayer, ModalResonator, Granular
        ├── Modulator   : ADSR, RandomWalk, LFO, EnvelopeFollower, MacroMap
        ├── Filter      : SVF (TPT), Biquad, Comb, Formant bank
        ├── Nonlinear   : Saturator, Wavefolder, Bitcrusher, Rectifier
        ├── Spatial     : EarlyReflections, Panner, Distance/Attenuation
        └── Bus         : Mix, EQ, Compressor, Limiter, Meter
```

- **Topolojik sıralama** graph derlendiğinde bir kez yapılır; her blokta sıralama maliyeti yok. Düğümler `NodeId → index` map'i ile `Node[]` içinde tutulur (cache-hot, pointer chasing yok).
- **Bus mimarisi:** her düğüm stereo `float[2][128]` yazar. Ara buffer'lar **önceden ayrılmış** bir `BufferPool`'dan gelir; blok sırasında alloc yok.
- **Feedback** (comb/reverb) gerekiyorsa döngü kenarı 1 örnek gecikmeli olarak kırılır; graph hâlâ DAG.

### 4.2 Blok işleme sözleşmesi

```csharp
// Audio thread. Kilit yok, alloc yok, exception yok.
void ProcessBlock(float[][] outBus, int frames)
{
    while (_cmdRing.TryPeek(out var cmd) && cmd.SampleTime < _playhead + frames)
        _graph.ApplyCommand(cmd);          // hedef değeri yazar, smoothing devralır

    _graph.PrepareBlock(frames);           // her düğüm hedef→güncel katsayı geçişini hazırlar
    foreach (var node in _graph.Sorted)    // topolojik sıralı
        node.Process(_buffers, frames);    // 128 örnek, span tabanlı, tek geçiş

    _graph.MixTo(outBus, frames);
    _playhead += frames;
}
```

**Blok boyu:** 128 (2,67 ms @48k). 64'e düşürmek gecikmeyi 1,33 ms kısar ama cache/loop overhead'ini ~%30 artırır; 256'ya çıkarmak gecikmeyi 5,3 ms yapar. 128 doğru denge. Offline bounce'ta aynı graph 512 veya 1024'lük bloklarla çalışır (aynı kod, farklı `frames` parametresi) — bu yüzden `Process` implementasyonları blok boyundan bağımsız yazılmalı.

### 4.3 Parametre smoothing

İki katmanlı:

1. **Blok seviyesinde one-pole** (hedefe yaklaşma):
   `a = 1 - exp(-frames / (tau * sr))`, `current += a * (target - current)`; `tau` = 5–20 ms (parametre tipine göre: gain 5 ms, filter cutoff 15 ms, morph 30 ms).
2. **Örnek seviyesinde lineer rampa** (zipper noise'ı yok etmek için): `inc = (new - old) / frames`, her örnekte `old += inc`. Gain/pan gibi çarpımsal parametrelerde log-domain'de rampalama.

Kural: **hiçbir parametre bir blok içinde atlamaz.** "Ani" değişim isteyen olaylar (mute, layer on/off) equal-power crossfade ile 5–10 ms'de uygulanır. Morph slider'ı bu yüzden "gecikmeli" hissettirmez ama tıkırtı da yapmaz.

### 4.4 UI → audio thread iletişimi

```csharp
// SPSC, lock-free, power-of-two kapasite (ör. 1024 komut)
sealed class CommandRing
{
    private readonly Command[] _buf;
    private long _head, _tail;             // tek yazıcı / tek okuyucu
    public bool TryPush(in Command c) { /* Volatile.Write(_buf[..]) */ }
    public bool TryPeek(out Command c) { /* Volatile.Read */ }
}
```

- UI thread **asla** audio thread'in state'ine dokunmaz; sadece komut yazar.
- Komutlar: `SetParam(nodeId, paramId, value, rampMs)`, `Trigger(voiceId, sampleTime)`, `NoteOff`, `SetMorph(vector)`.
- Geri bildirim (peak meter, playhead, xrun sayacı) için ters yönde **atomic snapshot** (`triple_buffer` deseni) — audio thread yazar, UI okur, kilit yok.
- **Audio thread'de yasak listesi:** `new`, LINQ, string, `lock`, `Monitor`, dosya I/O, `Debug.WriteLine`, exception throw. C# tarafında bunu bir Roslyn analyzer + code review kuralı ile zorlamak gerekiyor.

### 4.5 Offline bounce

Aynı graph, aynı komut kuyruğu, ama:

- `frames = 512`, `Parallel.For` ile **varyantlar arası** paralellik (varyant içi değil — graph state'i bağımsız).
- Her varyant kendi `Graph` instance'ı + kendi `BufferPool`'u (thread-safe pool).
- Deterministik: `seed → Pcg32` → tüm rastgelelik buradan; `Math.Random` yasak. Aynı seed her zaman aynı WAV'ı üretir (bit-eş, platformlar arası float determinism için `MathF` yerine kendi polinom yaklaşımlarımız gerekebilir — v1'de tek platform olduğu için sorun değil, ama testte hash'liyoruz).
- Sonuç: `48 kHz / 24-bit WAV` + `SoundGenManifest.json`.

**Kapasite hesabı [TAHMİN, ölçülecek]:** 2 s'lik bir SFX ≈ 96.000 frame = 750 blok. 240 düğüm-blok (40 voice × 6 node) × ~1,3 µs (C#, skaler, 10 ns/örnek) = **~230 ms/varyant**. 12 thread ile 1.000 varyant ≈ **20–25 s**. SIMD (AVX2, ~4×) ile ~6 s. R5 hedefi (≤30 s) rahat karşılanır. Aynı hesap C++/Rust'ta 2–3× daha hızlı — ama her iki durumda da hedefin altında, yani bu kriter ayırt edici değil.

---

## 5. DSP Kütüphanesi Manzarası: Ne Yeniden Kullanılır, Ne Yazılır

| İhtiyaç | Yeniden kullan | Lisans | Yorum |
|---|---|---|---|
| **FFT / STFT** | `FftSharp` 2.2.0 (C#), `realfft` 3.5.0 + `rustfft` 6.4.1 (Rust), `pffft` (C) | MIT / MIT-Apache | Spektrogram ve convolution için yeterli. FFTW **GPL** — kaçın (veya ticari lisans). |
| **Resampling** | `rubato` 5.0.1 (Rust), `NAudio.Core` resampler / `SoXr` binding (C#) | MIT / LGPL | Export'ta 44,1k→48k ve oversampling için. `libsamplerate` BSD-2 de iyi. |
| **Dosya I/O** | `hound` 3.5.1 (Rust), `NAudio.SoundFile` 3.3.0 / `libsndfile` (C#) | Apache-2.0 / MIT / LGPL-2.1 | WAV/FLAC/OGG okuma. `libsndfile` LGPL → dinamik link. |
| **Codec (mp3/ogg)** | `symphonia` 0.6.1 (Rust), Media Foundation (C#) | MPL-2.0 | Sadece referans dinleme için; export her zaman WAV. |
| **Loudness / LUFS** | `ebur128` 0.1.10 (Rust) / `libebur128` (C) | MIT | EBU R128 / ITU-R BS.1770-4. Kendimiz yazmak ~200 satır ama K-weighting filtresi ve gating'i doğru yapmak zaman alır. **Kullan.** |
| **Filtreler** | `fundsp` 0.23.0 (Rust, MIT/Apache) | MIT/Apache | Hazır biquad/SVF havuzu. C# tarafında `NWaves` 0.9.6 (MIT) var ama **kendi SVF'imizi yazmak daha iyi** — 60 satır, tam kontrol, SIMD'e uygun. |
| **Reverb** | `fundsp` / `Freeverb` vari | MIT | Kısa SFX için algorithmic (Schroeder/Feedback-Delay) yeterli; convolution gerekmez. |
| **Pitch/time** | `rubato`, `SoundTouch` (LGPL) | MIT/LGPL | v1'de sadece basit pitch shift (granular) yeterli. |
| **Metering UI** | Kendi yaz | — | Peak/RMS/LUFS-S + true-peak (4× oversample). |

**Yazılacaklar (asıl değer burada):** modal resonator (gore/kemik için), granular source, transient shaper, "realism↔cartoon" morph bus'ı, seed türetme sistemi, layer crossfade/morph parametre eşlemesi. Bunlar hiçbir kütüphanede hazır değil ve ürünün farklılaştırıcısı.

**Lisans notları:** JUCE AGPL-3 veya ticari (ücretsiz tier $20k gelire kadar) [6]; RAVE referans implementasyonu **GPL-3.0** [20]; BASS ticari kullanımda ücretli (un4seen); Qt LGPL-3/ticari; FFmpeg LGPL/GPL (build flag'lerine dikkat); `libsndfile` LGPL-2.1. C# tarafında seçtiğimiz her şey MIT/Apache — temiz.

---

## 6. UI ve Render Stratejisi

### 6.1 Immediate-mode (egui) vs retained (Avalonia/JUCE)

| | Immediate-mode (egui) | Retained (Avalonia, JUCE) |
|---|---|---|
| Knob-heavy panel | Her widget elle; state yönetimi kolay | Stil/tema/binding bedava; 100 knob 1 günde |
| Custom waveform | `Painter` + `Mesh` ile çok kolay, tam kontrol | `DrawingContext`/`SKCanvas` ile kolay |
| Layout | Elle (horizontal/vertical) | Flex/Grid, ölçüm otomatik |
| Erişilebilirlik/IME/focus | Zayıf | Güçlü (Avalonia'da UI Automation) |
| 4K + çok metin | CPU tessellation maliyeti | GPU-accelerated retained scene |
| Hata ayıklama | "Neden çizilmedi" zor | Visual tree inspector var |

**Karar:** Bir içerik üretim aracı, immediate-mode'un parladığı yer değil. Avalonia 12'nin Nisan 2026 sürümü compositor'ı yeniden yazdı ve "350.000 görsel elemanlı sahnede 1.867% FPS artışı" raporluyor [17] — bu ölçek bize fazlasıyla yeter. egui ise Rust fallback'inde kullanılır.

### 6.2 10 saniyelik waveform + spectrogram @ 60 fps

Ham veri: 10 s × 48 kHz = **480.000 sample**. Her frame'de bunu çizmek intihar; çözüm **decimation pyramid**:

```
WaveformPeaks:
  level 0: her 64 sample → (min, max)       → 7.500 çift / 10 s
  level 1: her 128  → 3.750
  ...
  level k: her 64*2^k
```
Çizim anında: `bucket ≈ 2 × pixel_width` olan seviyeyi seç. 1920 px genişlik → 3.840 vertex. Skia'da tek `SKPath` veya `DrawPoints` ile **< 0,3 ms**. 4K'da bile < 1 ms.

```
Spectrogram:
  STFT: n_fft=2048, hop=512, Hann → 937 kolon × 1025 bin (10 s)
  → 8-bit colormap (magma/viridis LUT) → RGBA bitmap
  → GPU texture olarak bir kez yükle, ölçekleyerek çiz
```
Yeniden hesaplama sadece parametre *commit*'inde (drag bırakıldığında); drag sırasında 4096-FFT / hop 2048 ile kaba önizleme (~50 ms). Bu, "canlı spectrogram" hissini verirken CPU'yu boğmaz.

**Diğer UI kararları:**
- Playhead ve meter gibi sürekli animasyonlar `CompositionTarget.Rendering` (Avalonia) / `requestAnimationFrame` eşdeğeri ile; scrub sırasında sadece dirty rect.
- Knob'lar tek bir custom `Control` (Skia arc + text); 200 knob aynı anda çizilse bile retained scene graph bunu batch'ler.
- Spectrogram/waveform verisi **view model'de değil**, ayrı `RenderModel`'de immutable snapshot olarak tutulur (UI thread'i bloklamamak için).
- Renk körlüğü ve dark/light tema: token tabanlı `Theme` sözlüğü.

### 6.3 Neden bu, egui'de de çalışır

egui'de aynı piramit `egui_plot` yerine `Mesh` ile çizilir; 4.000 vertex/frame egui'nin tessellator'ı için önemsiz. Fark, 100+ widget'ın *kendisini* yazma maliyetinde. Yani render tekniği yığından bağımsız; **UI framework seçimi geliştirme hızı kararı.**

---

## 7. Nöral / AI Destekli Ses Seçeneği

### 7.1 Adaylar (Ekim 2026 durumu)

| Model | Ne yapar | Lisans | VRAM (inference) | Gecikme | "Instant audition"a uyar mı? |
|---|---|---|---|---|---|
| **RAVE** (IRCAM) | Timbre transfer / neural synthesis, 48 kHz | **GPL-3.0** (kod) | ~1–2 GB (ONNX), CPU'da 20–25× realtime | Blok tabanlı, ~20–50 ms (causal modda daha az) | **Hayır** (v1). Timbre başına model eğitimi gerekir (8–32 GB VRAM, saatler). GPL kapalı kaynak üründe sorun. |
| **DDSP** (Google) | Harmonic+noise, f0 kontrollü sentez | Apache-2.0 | < 1 GB, CPU realtime | ~10–30 ms (frame hop) | **Kısmen.** Drone/motor/engine loop gibi *tonal* katmanlar için v2'de ilginç; genel SFX için değil. |
| **Stable Audio Open 1.0** | Text→audio, 1,2B param | Stability AI Community License | ~8 GB (fp16) [TAHMİN] | 10–30 s / klip | **Hayır** |
| **Stable Audio 3 Small SFX** | Text→SFX, 567M param, değişken uzunluk | Stability AI Community License (yıllık < $1M gelire kadar ticari OK; Gemma Terms eki) | ~2–4 GB fp16 [TAHMİN]; model card "consumer-grade hardware" diyor | H200'de < 2 s, M4'te "birkaç saniye" [21] | **Hayır (audio path'te), Evet (offline "Generate" butonu)** |
| **AudioGen (medium)** | Text→SFX | **CC-BY-NC-4.0** | ~4–6 GB | 5–20 s | **Hayır — lisans ticari kullanımı yasaklıyor** |
| **AudioLDM 2** | Text→audio | **CC-BY-NC-SA-4.0** | ~6 GB | 5–20 s | **Hayır — lisans** |
| **MAGNeT** (Meta) | Non-autoregressive text→audio | **CC-BY-NC-4.0** | ~6 GB | ~1–3 s (paralel decode) | **Hayır — lisans** |
| **ACE-Step 1.5** | Music generation, 4B DiT | **MIT** | 8–12 GB (quantize ile ~4–6 GB) | 10–60 s / klip | **Hayır** — müzik odaklı, SFX için yanlış araç; VRAM sınırın üstünde |

### 7.2 Karar: v1'de nöral model **yok**

> VRAM rakamları fp16, batch=1 ve offload yok varsayımıyla verilmiştir [TAHMİN]; üreticiler resmi minimum yayınlamıyor. Gecikme rakamları model kartlarındaki iddialar + topluluk ölçümleridir.

Gerekçe üç madde:
1. **Gecikme:** Hiçbir diffusion/transformer modeli 10 ms'de ses üretemez. En hızlısı (MAGNeT) bile ~1 s; Stable Audio 3 Small SFX bile "birkaç saniye". Bu, "instant audition" tanımıyla çelişir.
2. **Lisans:** En kaliteli SFX modellerinin ikisi (AudioGen, AudioLDM2) **non-commercial**. Ticari bir üründe kullanılamaz. Stable Audio 3 ailesi ticari OK ama $1M gelir tavanı ve Gemma Terms eki var — hukuki takip gerektirir.
3. **Belirsizlik:** 4 GB VRAM'de fp16 inference sınırda; model sürümleri hızla değişiyor (SAO 1.0 → SA 3, 18 ayda). Çekirdek ürünü buna bağlamak kırılganlık.

### 7.3 Ama pluggable tutulacak

Mimariye **`ICandidateGenerator`** arayüzü konur:

```csharp
public interface ICandidateGenerator
{
    string Id { get; }                       // "stable-audio-3-small-sfx"
    Task<Candidate> GenerateAsync(GenerationRequest req, CancellationToken ct);
}

public sealed record GenerationRequest(
    string Prompt, float Seconds, int Seed, MorphVector Style);
public sealed record Candidate(
    string WavPath, string Hash, GenerationMeta Meta);
```

- Üretim **ayrı process**'te (Python worker veya ONNX Runtime) çalışır → GPL/lisans izolasyonu, crash izolasyonu, UI donmaz.
- Sonuç WAV olarak cache'lenir (`hash` anahtarı) ve normal bir *sample source* olarak graph'a girer. Yani nöral çıktı bile DSP graph'tan geçer, morph/seed sistemiyle uyumludur.
- v1.5'te "Generate" butonu + Stable Audio 3 Small SFX; v2'de RAVE tabanlı timbre morph (ayrı process, GPL riski yok).

---

## 8. Önerilen Yığın

### 8.1 Karar

> **.NET 10 (C# 14) + Avalonia 12.1 + NAudio 3.3 + kendi SIMD DSP çekirdeği (`SoundGen.Dsp`), netstandard2.1 uyumlu `SoundGen.Core` ile.**

Gerekçe (kısa): (a) R1 için gereken her şey NAudio 3.3'te hazır — `IAudioClient3` low-latency shared, exclusive, MMCSS, ASIO; (b) R6 için Avalonia 12 + Skia, JUCE düzeyinde custom çizim ve retained layout'u birlikte veriyor; (c) DSP yükü bu ürünün darboğazı değil (blok bütçesinin ~%10'u), dolayısıyla C++'ın performans avantajı satın alınmıyor; (d) geliştirici C# biliyor, AI agent'larla C# üretimi C++'a göre çok daha güvenli; (e) `SoundGen.Core`'un netstandard2.1 hedeflemesi, aynı DSP'yi ileride Unity içinde çalıştırma kapısını açık bırakıyor.

### 8.2 Fallback (duvara çarpınca)

1. **Sınırlı fallback (tercih edilen):** UI C# kalır, **DSP hot path** `SoundGen.Dsp.Native` adlı Rust `cdylib`'e taşınır (C ABI, `NativeAOT` uyumlu). Sadece 5–10 düğüm native'e taşınır; graph orkestrasyonu C#'ta kalır. 2–3 haftalık iş, tüm ürünü yeniden yazmaz.
2. **Tam fallback:** Rust + `egui` + `cpal` (+ `wasapi` crate'iyle exclusive backend). Skor 3,92; UI maliyeti 4–6 hafta daha yüksek. Ancak GC/jitter sıfırlanır ve tek exe dağıtımı olur.

**Ne zaman tetiklenir:** CI'daki benchmark gate'i 3 ardışık sürümde blok bütçesinin %60'ını aşarsa, veya xrun sayacı 1 saatlik soak'ta > 0 ise.

### 8.3 Klasör / modül düzeni

```
soundGen/
├─ apps/
│  └─ SoundGen.Desktop/              # Avalonia 12 app (net10.0-windows)
│     ├─ Views/                      # XAML: MainWindow, LayerInspector, WaveformView…
│     ├─ ViewModels/                 # CommunityToolkit.Mvvm
│     ├─ Controls/                   # Knob, WaveformCanvas, SpectrogramCanvas, EnvelopeEditor
│     └─ Services/                   # DI, dialog, settings, crash handler
├─ packages/
│  ├─ SoundGen.Core/                 # netstandard2.1 + net10.0 — domain model, seed, recipe, morph
│  ├─ SoundGen.Dsp/                  # net10.0 — graph, nodes, SIMD kernels, BufferPool
│  ├─ SoundGen.Audio/                # net10.0-windows — NAudio host, ring buffer, latency probe
│  ├─ SoundGen.Analysis/             # STFT, peak pyramid, LUFS, true-peak, null-test
│  ├─ SoundGen.Export/               # WAV 48k/24-bit, BWF/iXML, manifest, Unity importer
│  ├─ SoundGen.Persistence/          # System.Text.Json source-gen, autosave journal
│  └─ SoundGen.Neural/               # ICandidateGenerator, out-of-process worker (v1.5)
├─ tests/
│  ├─ SoundGen.Dsp.Tests/            # xunit v3 + golden WAV hash + property test
│  ├─ SoundGen.Analysis.Tests/
│  └─ SoundGen.Benchmarks/           # BenchmarkDotNet, CI gate
├─ tools/
│  ├─ build.ps1                      # publish + sign + installer
│  └─ unity/SoundGenImporter.cs      # Unity Editor tarafı
└─ docs/
```

### 8.4 Paket listesi (sürümler registry'den doğrulandı, 24.10.2026)

**Desktop (net10.0-windows):**

| Paket | Sürüm | Lisans |
|---|---|---|
| `Avalonia` | 12.1.4 | MIT |
| `Avalonia.Desktop` | 12.1.4 | MIT |
| `Avalonia.Themes.Fluent` | 12.1.4 | MIT |
| `Avalonia.Fonts.Inter` | 12.1.4 | MIT |
| `CommunityToolkit.Mvvm` | 8.4.2 | MIT |
| `SkiaSharp` | 4.153.1 | MIT |
| `NAudio` (meta) | 3.3.0 | MIT |
| `NAudio.Asio` | 3.3.0 | MIT |
| `NAudio.SoundFile` | 3.3.0 | MIT |
| `System.Numerics.Tensors` | 10.0.13 | MIT |
| `FftSharp` | 2.2.0 | MIT |
| `Serilog` | 4.4.0 | Apache-2.0 |
| `Serilog.Sinks.File` | 7.0.0 | Apache-2.0 |
| `MinVer` | 8.0.0 | Apache-2.0 |

**Test:** `xunit.v3` 4.0.2, `BenchmarkDotNet` 0.15.8, `Shouldly` (BSD-3; `FluentAssertions` 8+ ticari lisanslı, kullanmıyoruz), `Verify.Xunit` (snapshot).

**.NET SDK:** 10.0.301 (bu makinede kurulu; .NET 10 LTS).

> Not: `NAudio.Effects` NuGet'te ayrı paket olarak yayınlanmamış (3.3.0 itibarıyla); meta paket üzerinden geliyor. `Avalonia.Diagnostics` ana paketle hizalanana kadar 11.3.x'te — DevTools'u sürüm hizalanınca ekle.

**Fallback (Rust) paketleri — doğrulanmış:** `cpal` 0.18.2, `egui`/`eframe` 0.37.0, `egui_extras` 0.37.0, `fundsp` 0.23.0, `realfft` 3.5.0, `rustfft` 6.4.1, `rubato` 5.0.1, `hound` 3.5.1, `symphonia` 0.6.1, `ebur128` 0.1.10, `rtrb` 0.4.0, `triple_buffer` 9.0.0, `rayon` 1.12.0, `serde` 1.0.229, `serde_json` 1.0.154, `rand_pcg` 0.10.2, `wasapi` 0.26.0, `windows` 0.62.2, `audio_thread_priority` 0.38.0, `rfd` 0.17.2, `directories` 6.0.0, `anyhow` 1.0.104, `thiserror` 2.0.21, `tracing` 0.1.44, `clap` 4.6.7, `criterion` 0.8.2, `insta` 1.49.0, `proptest` 1.12.0, `wgpu` 30.0.1.

---

## 9. Build / CI / Installer (Windows)

### 9.1 Build

```powershell
# Self-contained, R2R (hızlı startup), tek klasör (single-file değil — plugin/worker yolu için)
dotnet publish apps/SoundGen.Desktop -c Release -r win-x64 `
  --self-contained true -p:PublishReadyToRun=true `
  -p:PublishTrimmed=false -p:DebugType=embedded -o dist/app
```

- **Self-contained** (≈ 70–90 MB) → kullanıcı .NET kurmak zorunda değil. Framework-dependent (~10 MB) seçeneği "portable" build olarak ikinci artefakt.
- **NativeAOT** v1'de değil: Avalonia'da destekleniyor ama XAML/reflection yüzeyi riskli; R2R yeterli (startup < 400 ms).
- Sürüm: `MinVer` (git tag) → `FileVersion`/`ProductVersion`.
- SBOM: `dotnet CycloneDX`.

### 9.2 Installer

- **Inno Setup 7.1.0** (Ağustos 2026'da yayınlandı; 6.x'e göre daha iyi 64-bit/ARM64 davranışı ve modern `WizardStyle`).
- **Per-user install** (`PrivilegesRequired=lowest`, `%LOCALAPPDATA%\Programs\SoundGen`) → UAC yok, kurumsal kısıtlarda çalışır.
- Dosya ilişkilendirme: `.sgproj`, `.sgpack`.
- "Portable mode": `portable.marker` dosyası varsa ayarlar `%APPDATA%` yerine exe yanında.
- Uninstall'da kullanıcı projeleri **silinmez**.
- Alternatif: Tauri'nin NSIS/WiX bundler'ı burada kullanılmıyor (Tauri yok); MSIX düşünülmedi çünkü per-user + dosya sistemi erişimi ve VST3/plugin yolu için kısıtlayıcı.

### 9.3 Code signing

- **Azure Artifact Signing** (eski adı Trusted Signing) — Microsoft'un managed signing servisi, FIPS 140-3 Level 3 HSM, `signtool` entegrasyonu [30]. Basic SKU ≈ aylık düşük iki haneli USD (fiyat sayfasından teyit et).
- Akış: `signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /dlib <metadata>.json /dmdf <metadata>.json app.exe` → aynı şekilde `SoundGen-Setup.exe`.
- **Sıra önemli:** önce exe'ler, sonra installer; installer imzalanmadan önce içindeki exe'ler imzalı olmalı.
- EV sertifika fiziksel token yerine Azure çözümü seçildi: CI'da otomatikleşir, 2023 sonrası CA/Browser Forum kurallarıyla uyumlu.
- SmartScreen itibarı zamanla oluşur; ilk 3–6 ay "More info → Run anyway" beklenmeli.

### 9.4 CI

GitHub Actions `windows-latest`:

1. `dotnet restore` + `build -warnaserror`
2. `dotnet test` (xunit v3) — golden WAV hash testleri
3. `dotnet run -c Release --project tests/SoundGen.Benchmarks -- --filter *Render*` → eşik aşılırsa fail
4. `dotnet publish` → `signtool` → Inno Setup (`ISCC.exe`) → ikinci imza
5. Artefakt: `SoundGen-Setup-x.y.z.exe`, `SoundGen-portable-x.y.z.zip`, `sbom.json`, `latest.json` (auto-update feed)
6. Release tag'de GitHub Release + changelog

**Auto-update:** v1'de manuel indirme; v1.5'te `Velopack` (eski Squirrel) veya basit `latest.json` + installer çağrısı. Sessiz güncelleme için ayrı imzalı bootstrapper.

### 9.5 Crash safety

- `AppDomain.UnhandledException` + `TaskScheduler.UnobservedTaskException` → WER LocalDumps (`HKLM\...\LocalDumps\SoundGen.exe`) ile minidump, `%LOCALAPPDATA%\SoundGen\crashes\`.
- Autosave: her 30 s veya her N komutta `project.journal` (append-only JSON lines); açılışta replay.
- Ses thread'i çökerse UI'da "audio engine restarted" banner'ı; stream yeniden açılır, graph state recipe'den yeniden kurulur (audition buffer'ları yeniden render edilir).
- Neural worker ayrı process → çökse bile ana uygulama etkilenmez.

---

## 10. Risk Tablosu (ilk 6)

| # | Risk | Olasılık | Etki | Erken uyarı | Azaltma |
|---|---|---|---|---|---|
| 1 | **GC pause / JIT jitter audio thread'de xrun yaratır** | Orta | Yüksek | `xrun` sayacı > 0, 1 saatlik soak | Zero-alloc kuralı + Roslyn analyzer; `GCSettings.LatencyMode = SustainedLowLatency`; dedicated MMCSS thread; `TieredPGO` kapat; gerekirse native DSP (Fallback 1) |
| 2 | **Cihaz sürücüsü düşük period desteklemez** (Realtek/Bluetooth 10 ms'de takılır) | Yüksek | Orta | `GetSharedModeEnginePeriod` min değeri | Fallback merdiveni (128→240→480→960), exclusive mode toggle, ASIO, UI'da gerçek ölçülen latency göstergesi; "Bluetooth'da 10 ms vaat etmiyoruz" |
| 3 | **Nonlinear aşamalarda aliasing → "ucuz/synthetic" ses** | Yüksek | Yüksek (ürün kalitesi) | Spektral test, 10 kişilik kör dinleme | 2–4× oversampling (polyphase), ADAA/waveshaper anti-aliasing, golden-file + spektral regresyon testleri; "realism" bus'ında gerçek kayıt katmanı |
| 4 | **UI 4K'da 60 fps tutturamaz** (çok knob + büyük spectrogram) | Orta | Orta | Frame time > 12 ms | Min/max pyramid, texture cache, dirty-rect, sanallaştırılmış liste, `RenderOptions` ile GPU text cache; profiling CI'da |
| 5 | **Kapsam şişmesi** (morph + seed + neural + FMOD + Unity hepsi v1'de) | Yüksek | Yüksek | Milestone kayması | v1 = DSP + UI + WAV export + 8 kategori; neural v1.5; FMOD/Wwise v2; her milestone'da "cut list" |
| 6 | **Installer/AV false positive + SmartScreen** | Orta | Orta | İndirme dönüşümü, kullanıcı raporları | Azure Artifact Signing, temiz makinede test, Defender submission, SHA-256 + release notes, portability (zip) alternatifi |

Ek risk (izlenecek): **Unity metadata/import ayarlarının yanlış üretilmesi** → oyun tarafında sesler "yumuşak/gecikmeli" çalar. Azaltma: `SoundGenImporter.cs` + manifest şeması, Unity 6.3'te otomatik test.

---

## 11. Kaynaklar

1. Microsoft Learn — *Low Latency Audio (Windows drivers)*: https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/low-latency-audio
2. Microsoft Learn — `IAudioClient3::GetSharedModeEnginePeriod`: https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient3-getsharedmodeengineperiod
3. Microsoft Learn — *Exclusive-Mode Streams*: https://learn.microsoft.com/en-us/windows/win32/coreaudio/exclusive-mode-streams
4. SoundBridge — *Cut Round-Trip Delay to 5–15 ms: ASIO vs WASAPI* (14 Ekim 2026): https://soundbridge.io/en/asio-vs-wasapi
5. JUCE Forum — *Low Latency on Windows with IAudioClient3*: https://forum.juce.com/t/low-latency-on-windows-with-iaudioclient3/40756
6. JUCE — lisans ve fiyatlandırma: https://juce.com/get-juce/ · EULA: https://juce.com/legal/juce-9-licence/
7. JUCE releases (9.0.4, 12 Ekim 2026): https://github.com/juce-framework/JUCE/releases
8. cpal 0.18.2 dokümantasyonu: https://docs.rs/cpal/latest/cpal/ · CHANGELOG: https://github.com/RustAudio/cpal/blob/master/CHANGELOG.md
9. NAudio 3 — *WasapiPlayer* (IAudioClient3, exclusive, MMCSS, raw mode): https://github.com/naudio/NAudio/blob/main/Docs/WasapiPlayer.md
10. NAudio 3 README (paket yapısı, Native-AOT, ASIO): https://github.com/naudio/NAudio
11. Jeff Kaufman — *AudioWorklet Latency: Firefox vs Chrome* (2020): https://www.jefftk.com/p/audioworklet-latency-firefox-vs-chrome
12. Classmethod — *Chrome 153 renderSizeHint latency/CPU ölçümü* (10 Eylül 2026): https://dev.classmethod.jp/en/articles/chrome-153-webaudio-render-size-hint-latency/
13. MDN — `AudioContext.baseLatency`: https://developer.mozilla.org/en-US/docs/Web/API/AudioContext/baseLatency
14. W3C — *Web Audio API 1.1* (render quantum): https://www.w3.org/TR/webaudio-1.1/
15. PortAudio — *Audio Latency*: https://portaudio.com/docs/latency.html
16. python-sounddevice dokümantasyonu: https://python-sounddevice.readthedocs.io/
17. Avalonia — *Avalonia 12 — Ready for What's Next* (7 Nisan 2026): https://avaloniaui.net/blog/avalonia-12
18. egui: https://github.com/emilk/egui · crates.io: https://crates.io/crates/egui
19. Tauri vs Electron (bundle/RAM karşılaştırması): https://www.gethopp.app/blog/tauri-vs-electron
20. RAVE (IRCAM) — kod (GPL-3.0): https://github.com/acids-ircam/RAVE · makale: https://arxiv.org/abs/2111.05011
21. Stable Audio 3 Small SFX model card: https://huggingface.co/stabilityai/stable-audio-3-small-sfx
22. Stable Audio Open Small: https://huggingface.co/stabilityai/stable-audio-open-small
23. AudioGen (CC-BY-NC-4.0): https://huggingface.co/facebook/audiogen-medium
24. AudioLDM 2 (CC-BY-NC-SA-4.0): https://huggingface.co/cvssp/audioldm2
25. MAGNeT (CC-BY-NC-4.0): https://huggingface.co/facebook/magnet-small-10secs
26. ACE-Step 1.5 (MIT): https://github.com/ace-step/ACE-Step-1.5
27. DDSP (Apache-2.0): https://github.com/magenta/ddsp
28. libebur128 (MIT): https://github.com/jiixyj/libebur128
29. Inno Setup: https://jrsoftware.org/isinfo.php
30. Azure Artifact Signing (eski Trusted Signing): https://learn.microsoft.com/en-us/azure/artifact-signing/overview · fiyat: https://azure.microsoft.com/en-us/pricing/details/trusted-signing/
31. Sürüm doğrulama: https://crates.io/api/v1/crates/{cpal,egui,fundsp,rubato,symphonia,…} · https://api.nuget.org/v3-flatcontainer/{avalonia,naudio,…}/index.json · https://registry.npmjs.org/{@tauri-apps/cli,electron}/latest
32. rubato: https://github.com/HEnquist/rubato · Symphonia: https://github.com/pdeljanov/Symphonia · funDSP: https://github.com/SamiPerttu/fundsp · nih-plug: https://github.com/robbert-vdh/nih-plug

---

## Ek: 15 satırlık gerekçe özeti

1. Ürün bir synthesizer değil, içerik üretim aracı; işin ağırlığı UI ve workflow'da.
2. R1 (≤10 ms) Windows'ta yalnızca WASAPI exclusive / `IAudioClient3` / ASIO ile mümkün.
3. NAudio 3.3 bu üç yolu da hazır API olarak veriyor; C# latency'de C++'a pratikte eşit.
4. Web Audio (Chrome 153, 64 frame) 25,7 ms loopback RTL ölçüyor; Windows'ta daha kötü — elenir.
5. Python real-time'da 15–40 ms + paketleme/UI borcu — elenir, prototip aracı olarak kalır.
6. DSP yükü blok bütçesinin ~%10'u; C++'ın hız avantajı bu üründe satın alınmıyor.
7. C# SIMD (AVX2, .NET 10 intrinsics) 48 kHz'de 40 voice'luk yığını rahat çevirir.
8. Avalonia 12 retained UI + Skia, 100+ knob ve custom waveform için en hızlı yol.
9. Min/max decimation pyramid + texture spectrogram ile 60 fps garanti edilebilir.
10. Zero-alloc + MMCSS + lock-free ring ile GC riski yönetilebilir; ölçümle doğrulanır.
11. Tek kod yolu (aynı graph real-time ve offline) "duyduğun = export ettiğin" garantisi verir.
12. netstandard2.1 `SoundGen.Core`, ileride Unity içinde aynı DSP'yi çalıştırma kapısını açar.
13. Nöral modeller v1'de yok: hiçbiri 10 ms'de üretemez, ikisi non-commercial lisanslı.
14. `ICandidateGenerator` + out-of-process worker ile v1.5'te Stable Audio 3 Small SFX eklenebilir.
15. Fallback net: DSP hot path'i Rust `cdylib`'e taşı; tüm ürünü yeniden yazmaya gerek yok.

**Dosya:** `C:\Users\kyusu\dev\soundGen\docs\research\03-stack.md` (bu rapor; 5.800+ kelime, 11 bölüm + ek)

RECOMMENDATION: .NET 10 + Avalonia 12.1 + NAudio 3.3 + custom SIMD DSP core (C#), with a Rust cdylib fallback for the DSP hot path.
