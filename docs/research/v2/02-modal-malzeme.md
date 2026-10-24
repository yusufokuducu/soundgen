# 02 — Modal Resonator Bankları: Malzeme Fiziği ve Uygulama

> **Kapsam:** Her impact sesinin "gövdesi" olan modal bank. Teori (gerçek frekans oranları), 12 malzeme için
> koda doğrudan düşebilecek mod tabloları, uygulama detayları (excitation, pickup, nonlinearity, morph), cartoon
> eşlemesi ve mevcut `sg-dsp` koduyla fark analizi.
> **Hedef okuyucu:** yarın `sg-dsp/src/modal.rs`'i yeniden yazacak kişi.
> Tüm sayılar **48 kHz**, tüm T60 değerleri **saniye veya ms cinsinden mutlak** verilmiştir.

---

## 0. Teşhis: "Too synthetic" ne demek, tam olarak

Mevcut motor doğru resonator'ı yanlış veriyle besliyor. Sırayla, en çok duyulan suçludan:

| # | Kusur | Neden "synthetic" duyuluyor | Şu anki durum |
|---|---|---|---|
| 1 | **Pickup comb yok** | Her mod aynı genlikle duyuluyor. Gerçek nesnede vurma noktası ve dinleme noktası, modları `sin(kπx)` ile budar; spektrumda 3–5 adet keskin çukur oluşur. Bu çukurlar "vurulmuş nesne" algısının birinci bileşenidir. | `render.rs:210-221` aynı `exc`'i bütün modlara basıyor. |
| 2 | **Sönüm yasası düz** | `T60 ∝ 1/(1+0.35n)` → etkin `α ≈ 0.31`. Gerçek katılar 0.5–1.0. Sonuç: tiz modlar gereğinden 2–3× uzun kalıyor → "org/çan" hissi. | `modal.rs:111` |
| 3 | **İkizleşme (splitting) yok** | Simetrik nesnelerde her mod bir **doublet**'tir; 0.3–2 Hz beat üretir. Beat yoksa bank "filtrelenmiş gürültü" gibi durur, "çınlayan nesne" gibi değil. | `weapons.rs:32-48` tekli modlar |
| 4 | **Mode sayısı 4–5, HF yok** | Gerçek impact spektrumu 12–16 kHz'e uzanır. 5 modlu tablo 4 kHz'de bitiyor → "karton kutu". | `gore.rs:13`, `zombies.rs:27` |
| 5 | **Genliğe bağlı sönüm yok** | Sert vuruş = sadece daha yüksek ses. Gerçekte sert vuruş **daha parlak ve daha kısa** söner. Bu, oyuncunun vuruş şiddetini ayırt etmesinin tek yolu. | `render.rs:183-222` velosite kavramı yok |
| 6 | **Yanlış oran seti** | `gore.rs:13` `PLATE = [1, 1.594, 2.136, 2.296]` → bunlar **membran** oranları, plaka değil. Kafatası/kemik kırılması timpani gibi duyuluyor. | `gore.rs:13-15`, `zombies.rs:27` |
| 7 | **Excitation çok uzun** | `crates.rs:131` `exciter_ms = 10.0` → 10 ms temas süresi, 100 Hz'de kesen bir lowpass demek. Bu bir vuruş değil, sürtme. | `crates.rs:131` |

`01-dsp-sentez.md` §2.3 doğru yönde ama üç yerde yanlış: (a) "Dairesel plaka (free edge)" tablosu aslında membran
tablosu, (b) `T60_k = T60_0 (f_k/f_0)^-α` yasası verilmiş ama hiçbir preset'te uygulanmamış, (c) pickup comb
tek cümleyle geçiştirilmiş. Bu doküman üçünü de kapatıyor.

---

## 1. Teori

### 1.1 İdeal serbest-serbest çubuk (free-free beam, Euler–Bernoulli)

```
f_n = (β_n² / 2π) · sqrt(EI / (ρA L⁴))        β_n: cos β·cosh β = 1 kökleri
β_n ≈ (2n+1)π/2   (n ≥ 3)
f_n / f_1 = (β_n / β_1)²
```

| n | β_n | oran | not |
|---|---|---|---|
| 1 | 4.730041 | 1.000 | ilk bending |
| 2 | 7.853205 | 2.757 | |
| 3 | 10.995608 | 5.404 | |
| 4 | 14.137166 | 8.933 | |
| 5 | 17.278760 | 13.344 | |
| 6 | 20.420352 | 18.638 | |
| 7 | 23.561945 | 24.814 | |
| 8 | 26.703538 | 31.872 | |
| 9 | 29.845130 | 39.812 | |
| 10 | 32.986723 | 48.635 | |

**Ölçekleme:** `f_1 ∝ 1/L²`. 500 mm, Ø30 mm, 1.5 mm cidarlı çelik boru için `EI = 3.18e3 N·m²`, `ρA = 1.109 kg/m`
→ **f₁ = 762 Hz**. Aynı boru 300 mm'de 2117 Hz, 1.2 m'de 132 Hz. `size` knob'unun fiziksel temeli budur
(bkz. §3.5; kabuk modları ise `1/L` ile gider — gerçek nesneler ikisinin arasında, etkin üs ≈ 1.2–2.0).

Bu seriyi kullanan malzemeler: metal çubuk, kovan, tüfek namlusu, uzun ahşap kalas, anahtar, kilit dili.

### 1.2 İdeal dairesel membran (sabit kenar)

```
f_mn = (c / 2πa) · j_mn            j_mn = J_m Bessel fonksiyonunun n'inci sıfırı
f_mn / f_01 = j_mn / j_01
```

| # | (m,n) | j_mn | oran |
|---|---|---|---|
| 1 | (0,1) | 2.40483 | 1.000 |
| 2 | (1,1) | 3.83171 | 1.593 |
| 3 | (2,1) | 5.13562 | 2.136 |
| 4 | (0,2) | 5.52008 | 2.295 |
| 5 | (3,1) | 6.38016 | 2.653 |
| 6 | (1,2) | 7.01559 | 2.917 |
| 7 | (2,2) | 8.41724 | 3.500 |
| 8 | (0,3) | 8.65373 | 3.598 |
| 9 | (4,1) | 9.76102 | 4.059 |
| 10 | (1,3) | 10.17347 | 4.230 |

Kullanan: davul derisi, timpani, gerilmiş deri. **Zombie oyununda nadiren doğru cevap** — sadece gerçekten deri
olan yüzeyler (davul, gerdirme) için.

### 1.3 İdeal plaka (Kirchhoff, serbest kenar) — ve 01-dsp'teki hata

Plakada `f ∝ λ²`, membranda `f ∝ λ`. Yani plaka oranları ≈ membran oranlarının **karesi** (serbest kenarda
kenar kompliyansı bunu bir miktar aşağı çeker). Serbest dairesel plaka (Leissa tablosu, λ² değerleri):

| # | (s,n) | λ² | oran |
|---|---|---|---|
| 1 | (0,2) | 5.253 | 1.000 |
| 2 | (1,0) | 9.084 | 1.729 |
| 3 | (0,3) | 12.23 | 2.328 |
| 4 | (1,2) | 20.52 | 3.906 |
| 5 | (0,4) | 21.24 | 4.043 |
| 6 | (2,1) | 32.24 | 6.137 |
| 7 | (1,3) | 35.13 | 6.688 |
| 8 | (0,5) | 45.24 | 8.612 |

8'den sonrası için güvenilir tablo değeri vermiyorum; asimptotik olarak `f_n ≈ (n·π/a)²·sqrt(D/ρh)/(2π)` yani
`f ∝ n²` — son doğrulanmış orandan itibaren `(n/8)²` ile ekstrapole et.

Karşılaştırma (aynı ilk moda normalize):

| seri | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| Membran (sabit kenar) | 1.000 | 1.593 | 2.136 | 2.295 | 2.653 | 2.917 |
| Plaka, basit mesnetli (j²) | 1.000 | 2.539 | 4.561 | 5.269 | 7.039 | 8.511 |
| **Plaka, serbest kenar (gerçek)** | **1.000** | **1.729** | **2.328** | **3.906** | **4.043** | **6.137** |
| `gore.rs:13` `PLATE` | 1.000 | 1.594 | 2.136 | 2.296 | — | — |

**Aksiyon:** `gore.rs:13` ve `zombies.rs:27` membran setini kullanıyor. Bunlar plaka/kemik değil, davul.
Doğrusu §2.8'deki kemik tablosu.

Kullanan: cam kırığı, seramik, kemik parçası, metal levha, zil, tabla.

### 1.4 Gerilmiş tel (stiff string) — inharmonicity sabiti B

```
f_n = n · f_0 · sqrt(1 + B·n²)          B = π³·E·d⁴ / (64·T·L²)
temel kayması: f_1 = f_0·sqrt(1+B)     → 1200·log2(sqrt(1+B)) cent yukarı
```

`B` ölçülen aralıklar: piyano bas teli 1e-4…5e-4, orta oktav 1e-3…3e-3, en tiz oktav 1e-2…5e-2.
Gitar teli 1e-5…1e-4 (neredeyse harmonik).

B = 5e-4 (bas tel, +0.4 cent):

| n | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| oran | 1.0000 | 2.0015 | 3.0060 | 4.0150 | 5.0299 | 6.0522 | 7.0835 | 8.1250 | 9.1781 | 10.2444 |

B = 1e-2 (kısa/kalın tel, +8.6 cent):

| n | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| oran | 1.0000 | 2.0295 | 3.1165 | 4.2868 | 5.5624 | 6.9624 | 8.5022 | 10.1942 | 12.0482 | 14.0720 |

Zombie oyununda tel pek yok; ama `B`, **herhangi bir harmonik seti "bozmanın" en ucuz ve en doğru** yoludur:
`r_n = n·sqrt(1+B n²)`, tek parametre, tek `sqrt`. Cartoon morph'ta harmonik → inharmonik geçişi için de bunu kullan.

### 1.5 Gerçek nesneler neden sapıyor — ve ucuz taklidi

| Sapma | Fizik | Ölçü | Ucuz taklit |
|---|---|---|---|
| **Mode splitting** | Kusurlu simetri (et kalınlığı, darbe izi, kaynak) dejenere modu 2'ye (doublet) veya 3'e (triplet) ayırır | δ = (f_b − f_a)/f ≈ %0.1–2; çanlarda %5'e kadar | Modu `ratio·(1±δ/2)` ile **ikiye çoğalt**; amp 0.55/0.45, T60 %5–15 farklı, **zıt başlangıç fazı** |
| **Beat** | İkizler arası girişim | `f_beat = δ·f`; 480 Hz'de %1 → 4.8 Hz | Ayrı kod gerekmez — iki resonator otomatik yapar |
| **HF sıkışması** | Timoshenko düzeltmesi: kalın çubukta yüksek modlar teorik `β²`'den **aşağı** sapar | 6. modda %2–5 düşük | `ratio_n ← ratio_n · (1 − 0.004·n)` veya `ratio_n^0.97` |
| **Ampirik kayma** | Kütle eklenmesi (cıvata, boya, kir) | ±%0.5–3, mod başına | `× (1 + N(0, 0.008))` — asla %1'in altına inme (bkz. §4) |
| **Sönüm ayrışması** | Aynı `f`'te iki mod farklı mekanizmayla söner | — | İkizlere **farklı T60** ver; bu, çınlamayı "canlı" yapar |

**Kritik:** splitting, mistune'dan daha önemli. %2 mistune tek başına "akortsuz" duyulur; %1 splitting
"gerçek nesne" duyulur. İkisi birlikte: birinci doublet her zaman, gerisi sadece `f_k > 1.5 kHz` için.

### 1.6 Sönümleme: T60(f) yasası

```
T60(f) = T60_ref · (f / f_ref)^(−α)
Q(f)   = π·f·T60(f) / ln(1000) = π·f·T60(f) / 6.907755
       ⇒ Q(f) ∝ f^(1−α)
```

Üç rejim, ve neden önemli:

| α | T60(f) | Q(f) | Kulağa ne gelir | Kullanım |
|---|---|---|---|---|
| 0.0 | sabit | `∝ f` | org borusu, "synthesizer" — **en yapay seçim** | asla |
| 0.5 | `∝ f^-0.5` | `∝ √f` | ahşap, kemik, et | gerçekçi |
| 1.0 | `∝ 1/f` | **sabit** | karton, sıkı sönümlü | gerçekçi / cartoon için iyi default |
| 1.5 | `∝ f^-1.5` | `∝ f^-0.5` | boğuk, "tık" | concrete, kalın cidar |

**Loss factor → T60:** `T60 = ln(1000)/(π·f·η)`. Ama tablolanmış `η` **her zaman** birleşik nesnenin gerçek
sönümünden düşüktür — çünkü bağlantılar, montaj, radyasyon ve elle tutma baskın gelir. Ölçülmüş "montaj
faktörü" (T60_gerçek / T60_teorik):

| Malzeme | η (tablo) | T60_pred(1 kHz) | T60_ölçüm(1 kHz) | montaj faktörü |
|---|---|---|---|---|
| Çelik (kabuk) | 0.001 | 2199 ms | 1120 ms | **0.51** |
| Alüminyum levha | 0.002 | 1099 ms | 391 ms | **0.36** |
| Cam (şişe) | 0.001 | 2199 ms | 1444 ms | **0.66** |
| Meşe (kalas) | 0.015 | 147 ms | 94 ms | **0.64** |
| Çam (tahta) | 0.020 | 110 ms | 51 ms | **0.46** |
| Kemik (femur) | 0.020 | 110 ms | 96 ms | **0.87** |
| Seramik karo | 0.004 | 550 ms | 248 ms | **0.45** |
| Beton | 0.008 | 275 ms | 71 ms | **0.26** |
| ABS plastik | 0.025 | 88 ms | 96 ms | 1.09 |
| Karton | 0.15 | 15 ms | 6 ms | 0.38 |
| Et | 0.40 | 5.5 ms | 15 ms | 2.65 (η modeli geçersiz) |

**Kural:** `T60_efektif = 0.4–0.6 × T60(η)` çoğu katı için. §2'deki tablolar bu faktör **uygulanmış** hâlde.
Betonun 0.26'sı bir uyarıdır: betonun sönümü malzemeden değil **çatlak ve eklemden** gelir — bu yüzden beton
impact'i neredeyse tamamen gürültüdür, modal katman sadece "ağırlık" verir.

**T60 → katsayı (48 kHz):**

| T60 | r | eşdeğer −3 dB BW |
|---|---|---|
| 5 ms | 0.9716280 | 439.8 Hz |
| 10 ms | 0.9857119 | 219.9 Hz |
| 20 ms | 0.9928302 | 109.9 Hz |
| 50 ms | 0.9971259 | 44.0 Hz |
| 100 ms | 0.9985619 | 22.0 Hz |
| 300 ms | 0.9995204 | 7.33 Hz |
| 1 s | 0.9998561 | 2.20 Hz |
| 3 s | 0.9999520 | 0.73 Hz |

**Oyun klampı:** `T60 ≤ 3.0 s` (hero metal), `≤ 1.5 s` (arka plan). 12 modlu bir bankta 3 s'lik kuyruk
48 kHz'de 144 000 örnek × 12 mod = 1.7 M örnek-işlem; offline render'da sorun değil ama aynı klibi 64 voice
çalacaksan bütçeyi yer. Klampı **preset seviyesinde** yap, DSP'de değil.

### 1.7 Genliğe bağlı sönüm (velocity-dependent damping)

Ölçüm: sert vuruşta (a) T60 %20–45 kısalır, (b) 2–6 kHz enerjisi 3–8 dB artar, (c) mod oranları 0.2–0.5%
yukarı kayar (gerinim sertleşmesi). Yumuşak vuruşta tersi. Bu, "aynı sesin daha yükseği" hissini yok eden
tek mekanizma.

**Kararsızlığa girmeden uygulama — üç kural:**

1. **Kutup yarıçapını asla 1'e yaklaştırma.** Sadece **azalt**: `r_dyn = r_0 · (1 − λ·E^0.7)`, `λ ∈ [0, 0.35]`.
   `r_dyn < r_0 < 1` ⇒ her zaman kararlı. (Sistemin "daha uzun çınlaması" gerekmiyor; kısalması yeter.)
2. **Katsayıları blok başına güncelle, örnek başına değil.** 24 mod × 64 örnekte 1 `exp` + 1 `cos` =
   0.75 transcendental/örnek — pahalı. Bunun yerine `prepare()`'da `E ∈ {0, 0.33, 0.66, 1.0}` için
   **4 katsayı seti** hesapla ve `a1, a2`'yi lineer interpole et.
   *Kararlılık kanıtı:* 2-kutuplu resonator'ın kararlılık bölgesi `(a1,a2)` düzleminde
   `{(a1,a2) : a2 > |a1| − 1, a2 < 1}` üçgenidir ve bu küme **konveks**tir. İki kararlı nokta arasındaki
   doğru parçası tamamen bölge içinde kalır. Yani interpolasyon **koşulsuz kararlıdır** — bu, `r` üzerinden
   interpole etmekten farklı olarak ispatlanabilir bir garanti.
3. **Tıklama kontrolü:** 64 örnekte `Δr ≈ 0.35 × 2e-4 = 7e-5` → genlik süreksizliği < −80 dB. Ek yumuşatma gerekmez.

**Parlaklık (spectral tilt) için:** `A_k ← A_k · (1 + 1.8·v²·(f_k/1000)^0.5)` — sadece **excitation anında**,
yani ücretsiz. Sönüm tarafında yapmaya çalışma.

`v` (velocity) katman seviyesinde yoksa hiçbiri çalışmaz: `LayerKind::Modal`'a `velocity: f32` alanı şart (§5).

---

## 2. Mode Tabloları

**Okuma anahtarı.** Her tablo: `ratio` (f_k/f_0), `amp` (lineer, tablo içi max = 1.00), `f` (referans boyutta Hz),
`T60` (48 kHz, mutlak ms), `Q = πfT60/6.9078`. Tablolar **birleşik nesne** sönümünü içerir.
Farklı boyut için: `f_k = ratio_k · f0(size)` ve `T60_k = T60_0 · ratio_k^(−α)` (§3.5).
`amp` sütunu **pickup comb uygulanmamış** hâldir; comb §3.3'te ayrıca gelir.

### 2.1 Çelik varil / silah gövdesi (uzun, ince cidarlı silindir)

`f0 = 480 Hz`, `T60_0 = 1.90 s`, `α = 0.72`. Karakter: 1 = gövde bending ("donk"), 2 = kesit ovalling doublet,
3–12 = ring serisi (`f_n ∝ n(n²−1)/√(n²+1)`, çubuk serisine çok yakın: 1, 2.83, 5.42, 8.77, 12.86, 17.71).

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 1.00 | 480 | 1900.0 | 415 |
| 2 | 1.009 | 0.50 | 484 | 1887.8 | 416 |
| 3 | 2.756 | 0.62 | 1323 | 915.7 | 551 |
| 4 | 2.790 | 0.34 | 1339 | 907.7 | 553 |
| 5 | 5.404 | 0.40 | 2594 | 563.9 | 665 |
| 6 | 5.470 | 0.22 | 2626 | 559.0 | 668 |
| 7 | 8.933 | 0.25 | 4288 | 392.7 | 766 |
| 8 | 9.062 | 0.14 | 4350 | 388.6 | 769 |
| 9 | 13.340 | 0.16 | 6403 | 294.2 | 857 |
| 10 | 17.710 | 0.10 | 8501 | 239.9 | 928 |
| 11 | 23.530 | 0.07 | 11294 | 195.5 | 1004 |
| 12 | 30.100 | 0.04 | 14448 | 163.8 | 1076 |

Notlar: 2/1, 4/3, 6/5 … çiftleri doublet (δ ≈ 0.9%, beat 4.2 Hz @ 480 Hz). Namlu için `f0` 480→**900 Hz**
(ince, kısa) veya 480→**240 Hz** (kalın alıcı/susturucu gövdesi). Casing/kova için aynı tabloyu `f0` 2600 Hz
ve `T60_0 = 0.18 s` ile kullan.

### 2.2 Alüminyum levha / kutu (can)

`f0 = 1180 Hz`, `T60_0 = 0.34 s`, `α = 0.85`. Serbest plaka seti. Karakter: peak mod 3'te (2.75 kHz) — bu
"crick" sesidir, mod 1 değil.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 0.72 | 1180 | 340.0 | 183 |
| 2 | 1.729 | 0.44 | 2040 | 213.5 | 198 |
| 3 | 2.328 | 1.00 | 2747 | 165.8 | 207 |
| 4 | 3.906 | 0.30 | 4609 | 106.8 | 224 |
| 5 | 4.044 | 0.18 | 4772 | 103.7 | 225 |
| 6 | 6.137 | 0.20 | 7242 | 72.7 | 240 |
| 7 | 6.688 | 0.12 | 7892 | 67.6 | 243 |
| 8 | 8.612 | 0.09 | 10162 | 54.5 | 252 |
| 9 | 10.900 | 0.06 | 12862 | 44.6 | 261 |
| 10 | 13.200 | 0.04 | 15576 | 37.9 | 269 |

Kalın levha (2 mm+) için `α` 0.85→0.95 ve `f0` ×0.7. İçi boş kutu için: aynı set, ama `T60` ×1.6 ve ek olarak
**60–140 Hz arası bir Helmholtz modu** (`Q ≈ 8`, T60 120 ms) — bu, kutunun "içi boş" hissidir.

### 2.3 Sert ahşap kalas (meşe sandık tahtası)

`f0 = 158 Hz`, `T60_0 = 0.26 s`, `α = 0.55`. 1 = bending, 2 = torsional (geniş kalasta çok belirgin,
1.58–1.70× arası), 3+ = çubuk serisi hafif sıkışmış.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 0.85 | 158 | 260.0 | 18.7 |
| 2 | 1.630 | 0.55 | 258 | 198.7 | 23.3 |
| 3 | 2.712 | 1.00 | 428 | 150.2 | 29.3 |
| 4 | 5.240 | 0.42 | 828 | 104.6 | 39.4 |
| 5 | 8.550 | 0.24 | 1351 | 79.9 | 49.1 |
| 6 | 12.620 | 0.14 | 1994 | 64.5 | 58.5 |
| 7 | 17.200 | 0.08 | 2718 | 54.4 | 67.2 |
| 8 | 22.300 | 0.05 | 3523 | 47.1 | 75.5 |
| 9 | 28.300 | 0.03 | 4471 | 41.4 | 84.1 |

Torsional oranı kalasın en/boy oranına bağlı: dar (1:4) → 1.63, geniş (1:6) → 1.79. Ahşabın "tok" hissi
2–3 kHz'deki boşluktan gelir: mod 5 (1.35 kHz) ile mod 6 (1.99 kHz) arasında 640 Hz'lik bir delik var, bu
gerçek — doldurma.

### 2.4 Yumuşak ahşap (çam tahtası)

`f0 = 124 Hz`, `T60_0 = 0.185 s`, `α = 0.62`. Aynı iskelet, daha sönümlü, daha kısa, tiz daha hızlı ölü.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 0.80 | 124 | 185.0 | 10.4 |
| 2 | 1.580 | 0.45 | 196 | 139.3 | 12.4 |
| 3 | 2.690 | 1.00 | 334 | 100.2 | 15.2 |
| 4 | 5.200 | 0.36 | 645 | 66.6 | 19.5 |
| 5 | 8.440 | 0.18 | 1047 | 49.3 | 23.5 |
| 6 | 12.400 | 0.10 | 1538 | 38.8 | 27.2 |
| 7 | 16.900 | 0.05 | 2096 | 32.1 | 30.5 |
| 8 | 21.900 | 0.03 | 2716 | 27.3 | 33.7 |

Çam vs meşe ayrımı tek parametrede: `α` (0.62 vs 0.55) + `T60_0` (0.185 vs 0.26) + Q@1k (23 vs 43).
Bunu iki ayrı tablo yapmak yerine tek tablo + `damping_scale` ile de yapabilirsin, ama oran seti farklı
(1.58 vs 1.63) → ayrı tablo daha temiz.

### 2.5 Cam: şişe / kırık

**Şişe** — `f0 = 520 Hz`, `T60_0 = 1.90 s`, `α = 0.42` (camın tizi *yavaş* söner; bu, camı metalden ayıran şey).
Mod 1 = Helmholtz hava modu (0.42×) — **ayrı mekanizma**: hava boşluğu viskoz sınır tabakasıyla söner,
`Q = 15–40`. Bu yüzden `α` yasası **bu satıra uygulanmaz**; tabloda 220 ms elle konmuş değerdir.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 0.420 | 0.30 | 218 | **220** | 21.8 |
| 2 | 1.000 | 1.00 | 520 | 1900.0 | 449 |
| 3 | 1.006 | 0.62 | 523 | 1895.2 | 451 |
| 4 | 2.830 | 0.55 | 1472 | 1227.4 | 822 |
| 5 | 2.860 | 0.35 | 1487 | 1222.0 | 827 |
| 6 | 5.420 | 0.28 | 2818 | 934.3 | 1198 |
| 7 | 8.770 | 0.18 | 4560 | 763.3 | 1583 |
| 8 | 12.900 | 0.10 | 6708 | 649.1 | 1980 |
| 9 | 17.700 | 0.06 | 9204 | 568.3 | 2379 |

"Bwoo" sesi o 218 Hz'lik moddur ve onsuz şişe, şişe gibi durmuyor. Çan (bell) için bu mod `0.5` (hum tone),
`amp 0.55` olur ve tablo 4–5 moda düşer.

**Kırık** — `f0 = 1800 Hz`, `T60_0 = 0.095 s`, `α = 0.50`, plaka seti. Küçük, serbest, yüksek Q.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 0.75 | 1800 | 95.0 | 78 |
| 2 | 1.729 | 1.00 | 3112 | 72.2 | 102 |
| 3 | 2.328 | 0.85 | 4190 | 62.3 | 119 |
| 4 | 3.906 | 0.42 | 7031 | 48.1 | 154 |
| 5 | 4.044 | 0.30 | 7279 | 47.2 | 156 |
| 6 | 6.137 | 0.22 | 11047 | 38.3 | 193 |
| 7 | 6.688 | 0.14 | 12038 | 36.7 | 201 |
| 8 | 8.612 | 0.09 | 15502 | 32.4 | 228 |

Kırık için `T60` **boyuta bağlı**: 5 mm parça 40 ms, 40 mm parça 150 ms. `T60_0 ∝ size^0.7`.

### 2.6 Seramik karo

`f0 = 1080 Hz`, `T60_0 = 0.235 s`, `α = 0.68`. Karakter: 1.5–4.5 kHz arası sıkışmış, "clink" — metalden
daha az Q, camdan daha az tiz.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 0.85 | 1080 | 235.0 | 115 |
| 2 | 1.660 | 1.00 | 1793 | 166.5 | 136 |
| 3 | 2.280 | 0.78 | 2462 | 134.2 | 150 |
| 4 | 3.720 | 0.40 | 4018 | 96.2 | 176 |
| 5 | 4.150 | 0.32 | 4482 | 89.3 | 182 |
| 6 | 6.020 | 0.20 | 6502 | 69.3 | 205 |
| 7 | 6.550 | 0.13 | 7074 | 65.5 | 211 |
| 8 | 8.400 | 0.09 | 9072 | 55.3 | 228 |
| 9 | 10.500 | 0.05 | 11340 | 47.5 | 245 |
| 10 | 12.800 | 0.03 | 13824 | 41.5 | 261 |

Kalın fayans (8 mm) → `f0` ×1.35, `α` 0.68→0.75. İnce karo (4 mm) → `f0` ×0.8, `T60` ×1.4.

### 2.7 Kemik (femur benzeri, içi boş kalın cidar)

`f0 = 860 Hz`, `T60_0 = 0.105 s`, `α = 0.60`. 1 = bending, 2 = ovalling, 3/4 ve 6/7 = doublet'ler.
Kemiğin imzası: 1.4–2.4 kHz kümesi + 3.4 kHz'de güçlü tek mod + çok hızlı HF ölümü.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 0.80 | 860 | 105.0 | 41 |
| 2 | 1.620 | 1.00 | 1393 | 78.6 | 50 |
| 3 | 2.720 | 0.55 | 2339 | 57.6 | 61 |
| 4 | 2.780 | 0.35 | 2391 | 56.9 | 62 |
| 5 | 3.940 | 0.45 | 3388 | 46.1 | 71 |
| 6 | 5.310 | 0.28 | 4567 | 38.6 | 80 |
| 7 | 5.440 | 0.18 | 4678 | 38.0 | 81 |
| 8 | 7.900 | 0.12 | 6794 | 30.4 | 94 |
| 9 | 9.200 | 0.07 | 7912 | 27.7 | 100 |
| 10 | 11.600 | 0.04 | 9976 | 24.1 | 110 |

Kuru kemik vs yaş kemik: yaş için `T60` ×0.6, `α` 0.60→0.75, amp `>3 kHz` ×0.5. Kafatası için `f0` ×0.75
(daha büyük, daha ince cidar) ve mod 2 amp 1.2 (kubbe "bom" yapar).

### 2.8 Et / kas kütlesi (ağır sönümlü, düşük Q)

`f0 = 92 Hz`, `T60_0 = 0.048 s`, `α = 0.50`. Q = 2–8. **Bu tablo sesin sadece %25–35'i.** Et impact'inin
geri kalanı noise + granular (§2.13'teki gibi). Modal bank burada sadece "kütle" verir.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 1.00 | 92 | 48.0 | 2.0 |
| 2 | 1.850 | 0.55 | 170 | 35.3 | 2.7 |
| 3 | 3.200 | 0.30 | 294 | 26.8 | 3.6 |
| 4 | 5.100 | 0.16 | 469 | 21.3 | 4.5 |
| 5 | 7.400 | 0.09 | 681 | 17.6 | 5.5 |
| 6 | 10.500 | 0.05 | 966 | 14.8 | 6.5 |
| 7 | 14.200 | 0.03 | 1306 | 12.7 | 7.6 |

**Uyarı:** `T60_0 = 48 ms` bir *vuruş* içindir. Etin "squish" bileşeni modal değil: `Svf(BP, Q=2..4)` sweep
+ granular. Et impact'inde modal katman −8…−12 dB'de kalmalı.

### 2.9 Beton / taş

`f0 = 155 Hz`, `T60_0 = 0.38 s`, `α = 0.90`. Q ≈ 27–35, frekansa neredeyse sabit. Betonun sönümü malzemeden
değil eklemden gelir → spektrum düz, modlar zayıf, gürültü baskın.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 1.00 | 155 | 380.0 | 26.8 |
| 2 | 1.520 | 0.62 | 236 | 260.7 | 27.9 |
| 3 | 2.310 | 0.48 | 358 | 178.9 | 29.1 |
| 4 | 3.140 | 0.30 | 487 | 135.7 | 30.0 |
| 5 | 4.420 | 0.22 | 685 | 99.7 | 31.1 |
| 6 | 6.050 | 0.14 | 938 | 75.2 | 32.1 |
| 7 | 8.100 | 0.09 | 1256 | 57.8 | 33.0 |
| 8 | 10.600 | 0.05 | 1643 | 45.4 | 33.9 |
| 9 | 13.800 | 0.03 | 2139 | 35.8 | 34.8 |

Büyük döşeme (3 m) → `f0` 60–90 Hz, `T60_0` 0.55 s. Küçük moloz → `f0` 400–900 Hz, `T60_0` 0.08 s.

### 2.10 Karton / kâğıt

`f0 = 430 Hz`, `T60_0 = 0.013 s`, `α = 1.00` → **sabit Q = 2.5**. Neredeyse mod değil, şekillenmiş gürültü.
Bu tabloyu kullanmak yerine çoğu durumda `velvet noise → BPF` daha iyi. Ama kutu/vuruş için işe yarar.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 1.00 | 430 | 13.0 | 2.5 |
| 2 | 1.900 | 0.45 | 817 | 6.8 | 2.5 |
| 3 | 3.300 | 0.28 | 1419 | 3.9 | 2.5 |
| 4 | 5.200 | 0.16 | 2236 | 2.5 | 2.5 |
| 5 | 7.600 | 0.09 | 3268 | 1.7 | 2.5 |
| 6 | 10.400 | 0.05 | 4472 | 1.2 | 2.5 |
| 7 | 14.000 | 0.03 | 6020 | 0.9 | 2.5 |

**Karışım oranı:** karton = %60 noise (velvet, 1800 pps) + %40 modal. Modal'ı tek başına kullanırsan
"plastik tık" duyulur.

### 2.11 Plastik kabuk (ABS/PVC, 2 mm)

`f0 = 720 Hz`, `T60_0 = 0.125 s`, `α = 0.80`. Karakter: 1.2 kHz'de baskın mod (içi boş "tok"), tiz hızlı ölü.
Plastiği ahşaptan ayıran: aynı `T60`'ta **daha az tiz, daha çok orta** — yani `α` daha yüksek, `f0` daha yüksek.

| # | ratio | amp | f (Hz) | T60 (ms) | Q |
|---|---|---|---|---|---|
| 1 | 1.000 | 0.85 | 720 | 125.0 | 41 |
| 2 | 1.720 | 1.00 | 1238 | 81.0 | 46 |
| 3 | 2.350 | 0.70 | 1692 | 63.1 | 49 |
| 4 | 3.880 | 0.35 | 2794 | 42.3 | 54 |
| 5 | 4.100 | 0.25 | 2952 | 40.4 | 54 |
| 6 | 6.100 | 0.16 | 4392 | 29.4 | 59 |
| 7 | 6.720 | 0.10 | 4838 | 27.2 | 60 |
| 8 | 8.500 | 0.07 | 6120 | 22.6 | 63 |
| 9 | 10.800 | 0.04 | 7776 | 18.6 | 66 |
| 10 | 13.500 | 0.03 | 9720 | 15.6 | 69 |

Boş bidon/damar → `f0` 260–400 Hz, `T60_0` 0.30 s, ek Helmholtz modu. Sert plastik (PC) → `α` 0.65, `T60` ×1.5.

### 2.12 Su / kan sıçraması — **modal değil**

Sıçrama bir yapı değil, bir **olay**. Modal bank burada yanlış araç; kullanırsan "plastik su" elde edersin.
Doğru model: **Minnaert kabarcığı** (tek serbestlik dereceli, ama frekansı boyuta bağlı ve sönümü viskoz).

```
f_bubble = (1/2πR)·sqrt(3γP/ρ) = 3.283 / R        [R metre, f Hz]
γ=1.4, P=101.3 kPa, ρ=1000 kg/m³        Q = 10…40 (viskoz + termal)
T60 = ln(1000)·Q / (π·f)
```

| R (mm) | 0.3 | 0.5 | 1 | 2 | 3 | 5 | 8 | 12 |
|---|---|---|---|---|---|---|---|---|
| f (Hz) | 10943 | 6566 | 3283 | 1642 | 1094 | 657 | 410 | 274 |
| T60 (ms, Q=20) | 4.0 | 6.7 | 13.4 | 26.8 | 40.2 | 67.0 | 107 | 161 |

**Sıçrama reçetesi (modal bank yerine):**

| Katman | Üretim | Parametre |
|---|---|---|
| SPLAT | `Blast(td=2..6 ms)` → `LP, fc 6k→700 Hz, 40..90 ms` | kütle; büyük gore'da `td` uzun |
| BUBBLES | 3–12 adet **kısa decaying sine**, `f = 3.283/R`, `R ~ log-uniform 0.4..8 mm`, Poisson 0..180 ms | sıçramanın "cıvıltısı"; asıl su kimliği |
| SQUELCH | `Osc(sine, 240 Hz)` → `PitchEnv(1.8×→1.0×, τ=35 ms)` + `AM(22 Hz, %45)` | viskoz akış |
| WET TAIL | `Noise(pink)` → `BPF 900 Hz Q=1.2` sweep ↓ 380 Hz, 120 ms | ıslaklık |
| DRIPS | 1–3 adet, 200–900 ms sonra, aynı bubble üreteci, −18…−26 dB | "damlıyor" |

**Kural:** gore sıçramasında modal katman **sadece** "kemik/kova çarpması" için (varsa). Sıvının kendisi
hiçbir zaman modal olmamalı.

---

## 3. Uygulama

### 3.1 Banka mimarisi: paralel 2-pole vs. FDN

**Karar: 8–24 mod için paralel 2-pole resonator. FDN değil.**

| | Paralel 2-pole ×N | FDN (N delay line) |
|---|---|---|
| Her moda bağımsız `f`, `T60` | ✅ doğrudan | ❌ **imkânsız** — sönüm döngü kazancına bağlı, tüm gecikmelerle paylaşılır |
| Mod oranları kesin mi | ✅ | ❌ matrisin özdeğerleri; tasarım gereği dağınık |
| Örnek başına işlem (N=24) | **~72 op** | ~265 op (24×7 interp okuma + Householder 25 + 24 loss filtresi + çıkış) |
| Durum belleği | 24×3 float = 288 B | 24 gecikme hattı (birkaç KB) + matris |
| Zamanla değişen `T60` | ✅ katsayı değiştir | ❌ tüm döngüyü yeniden tasarla |
| "Yoğunluk" (difüz kuyruk) | ❌ | ✅ |

FDN'in tek üstünlüğü **difüzite**, yani reverb. Bir varilin "bu varil" olması için gereken şey difüzite değil,
**belirli oranlar** — bu yüzden resonator doğru araç. 24 modun üstüne çıkarsan (çan, büyük sac) FDN hibridi
düşünülebilir, ama o zaman da modal bankı bırakmıyorsun, üstüne ekliyorsun.

**CPU (48 kHz, tek çekirdek):**

| Mod | op/örnek | Mop/s (voice) | çekirdek % (3 GHz, skaler) | AVX2 8-lane |
|---|---|---|---|---|
| 8 | 24 | 1.15 | ~0.02% | ~0.004% |
| 12 | 36 | 1.73 | ~0.03% | ~0.006% |
| 16 | 48 | 2.30 | ~0.04% | ~0.008% |
| 24 | 72 | 3.46 | ~0.06% | ~0.012% |

Seri bağımlılık notu: her modun özyinelemesi `y[n]`'e bağlı → **bir mod içinde vektörleştirilemez**. Vektörleme
**modlar arasında** olur. Bu yüzden veri yerleşimi **SoA** olmalı:
`a1[8], a2[8], y1[8], y2[8]` → 8 mod tek AVX register'ında, 24 mod 3 grup. AoS (`struct`) 3–4× yavaş
(ve `sg-dsp` şu an AoS: `Vec<ModeState>`, `modal.rs:62-71`).

Buna karşılık **24 voice × 24 mod** = 576 mod = ~1.7% çekirdek. Bütçe rahat.

### 3.2 Excitation enjeksiyonu

**İki mekanizma, ikisi de gerekli:**

**(A) Vuruş = başlangıç koşulu (initial condition), kuvvet değil.**
Bir resonator'a impuls basmak yerine, modu doğrudan "çalışır hâlde" başlat. `y[n] = A·r^n·cos(ωn + φ)` için:

```rust
y1 = A * cos(φ);
y2 = (A / r) * cos(φ - ω);     // bir örnek geri kaydırılmış
```

4 op, sadece trigger anında. Kazançlar:

- **Kontak süresi (T_c) bir lowpass olarak mode genliğine katlanır** — ayrı filtre yok:
  `|H(T_c, f)| = |sin(π f T_c) / (π f T_c)|`
- Pickup comb'u **kesin** olarak uygulayabilirsin (aşağıda), çünkü `A_k`'yi sen belirliyorsun
- DC/HP/denormal sorunu yok, `x[n]` hiç hesaplanmıyor
- `φ` serbest → modlar arası faz dağılımını kontrol edersin (gerçekte `φ ≈ 0`, ama hafif yayılmış `φ` "temas
  yüzeyi geniş" hissi verir; `φ_k = N(0, 0.15·π)`)

Kontak süresi tablosu (yarım-sinüs darbe, ilk spektral sıfır `1/T_c`):

| Yüzey | T_c | −6 dB köşe | Örnek |
|---|---|---|---|
| metal–metal | 0.15 ms | 4.0 kHz | kovan, sürgü, kilit |
| metal–ahşap | 0.4 ms | 1.5 kHz | dipçik, sandık |
| ahşap–ahşap | 0.8 ms | 0.75 kHz | kalas, kasa |
| kemik–kemik | 0.25 ms | 2.4 kHz | kemik kırılması |
| et–et | 4.0 ms | 250 Hz | yumruk, ısırma |
| karton | 3.0 ms | 330 Hz | kutu |

**(B) Sürekli uyarım (sürtme, kayma, gıcırtı, yuvarlanma) = gerçek giriş.**
Burada mevcut davranış (`bank.process(exc)`) doğru — StickSlip çıkışını veya noise burst'ünü banka bas.
Bu durumda `T_c` filtresi gerekmez, çünkü kaynak zaten gürültü.

**Gain normalizasyonu (şart).** `A_k`'yi ölçekleyip bırakmak, mode sayısı değişince seviyeyi değiştirir:
5 modlu tablo `Σamp = 2.26`, 12 modlu çelik tablosu `Σamp = 3.85` → **+4.6 dB**. Coherent tepede daha da kötü.
`prepare()`'da ölç:

```
E = Σ_k A_k² · (1/(1 − r_k²))          // IR enerjisi, kapalı form
ölçek = 1 / sqrt(E)                     // hedef: birim enerji
```

`1/(1−r²)` yerine `T60·fs/6.9078/2` de kullanılabilir (aynı şey, daha hızlı). Bu tek satır, kütüphane
genelinde loudness tutarlılığını çözer — şu an her preset `gain_db`'yi elle trim ediyor.

### 3.3 Output pickup: "dinleme noktası" comb'u

Bir çubuk/plaka yüzeyinde, `x_s` noktasından vurulup `x_l` noktasından dinlenirse mod genliği:

```
A_k ∝ |sin(k π x_s)| · |sin(k π x_l)|          (çubuk/tel, k = 1..N)
```

Plaka için `|J_m(j_mn x)|`, ama `sin` yaklaşımı 2–4 kHz üstünde yeterince iyi ve 2 op.

**Neden bu, gerçekçiliğin ikinci en önemli bileşeni:** çarpım, spektrumda **düzenli ama periyodik olmayan**
çukurlar üretir. `x_s = 0.19`, `x_l = 0.34` için çukurlar:

| sıfır kaynağı | k değerleri | yakın mod |
|---|---|---|
| `sin(kπ·0.19)` | 5.26, 10.5 | 5, 10 |
| `sin(kπ·0.34)` | 2.94, 5.88, 8.82, 11.76 | 3, 6, 9, 12 |

Birleşik: mod 5–6, 9, 12 ciddi budanır. Bu, "karton kutu" ile "gerçek kalas" arasındaki farkın ta kendisi.
Mevcut kodda bu **hiç yok** → bütün modlar eşit genlikte → filtrelenmiş gürültü.

**Varsayılanlar:**

| Nesne tipi | `x_s` | `x_l` | `depth` |
|---|---|---|---|
| çubuk / kalas / namlu | 0.18 ± 0.06 | 0.31 ± 0.08 | 0.85 |
| plaka / sac / karo | 0.22 ± 0.08 | 0.38 ± 0.10 | 0.80 |
| kabuk / varil (silindir) | 0.25 ± 0.05 | 0.50 ± 0.12 | 0.70 |
| kova / küçük metal | 0.30 | 0.45 | 0.90 |
| et / kütle | 0.5 | 0.5 | **0.15** (et budanmaz) |
| cartoon | 0.5 | 0.5 | **0.10** (§4) |

**Zorunlu koruma:** `A_k = amp_k · (1 − depth + depth·|sin|·|sin|)`. `depth < 1` olduğu için tam sıfır oluşmaz
(gerçek nesnelerde de olmaz — asimetri). `x_s == x_l` yapma: comb karesi alınır, iki kat derin çukur → "woodblock".

Kod:

```rust
#[inline]
fn comb(k: usize, xs: f32, xl: f32, depth: f32) -> f32 {
    let a = (core::f32::consts::PI * k as f32 * xs).sin().abs();
    let b = (core::f32::consts::PI * k as f32 * xl).sin().abs();
    1.0 - depth + depth * a * b
}
```

### 3.4 Nonlinear coupling: "clang" growl

Sert vuruşta metal, doğrusal olmayan temas ve geometrik sertleşme yüzünden **intermodülasyon** üretir:
`f_a ± f_b` yan bantları, gerçek modların *arasında* düşer. Kulak bunu "metal gerildi / bağırdı" diye okur.
Doğrusal bankta bu hiç olmaz — ne kadar sert vurursan sadece o kadar yüksek ses.

**Ucuz ve koşulsuz kararlı şema (feed-forward, hedefe geri besleme yok):**

```rust
// 1 kez per örnek, tüm bank için:
let s = 0.6 * y[0] + 0.4 * y[1];          // 2 sürücü mod (en düşük ikisi)
let d = s - s * s * s * (1.0 / 3.0);      // yumuşak kubik, |d| ≤ 1
let inj = mu * d;                          // mu = 0..0.45

// hedef modlara (ratio 5.4 ve 8.9) girişe ekle:
y[4] += inj * 0.7;
y[6] += inj * 0.4;
```

- **Maliyet:** 1 add + 3 mul + 1 çıkarma = 5 op **tüm bank için** (mod başına değil). `tanh` kullanma
  (15–30 cycle); `s − s³/3` yeterli ve sınırlı.
- **Kararlılık:** `|d| ≤ 1` → enjeksiyon sınırlı; alıcı resonator'lar `r < 1` → sınırlı giriş, sınırlı çıkış.
  Sürücüden hedefe geri besleme **yok** → sistem feed-forward → **her `mu` için kararlı**. İspat gerekmiyor,
  yapı gereği.
- **`mu` seyri:** `mu = mu_max · v^1.5` (v = velocity). `mu_max`: metal 0.45, seramik 0.25, ahşap 0.12,
  kemik 0.20, cam 0.15, et 0.0.
- **Ne üretir:** 480 + 1323 Hz → 843 ve 1803 Hz. Bunlar tabloda yok → sert vuruşa özel "fazladan metallik
  partial"lar. Tam istediğimiz şey.
- **Growl (sürekli):** aynı enjeksiyonu StickSlip/noise kaynağıyla sür → gıcırtının "gıcırdaması".

**İkinci mekanizma — sönüm modülasyonu (bedava):** `r_eff = r · (1 − 0.25·|s|)`. Sadece 1 mul; mod
"nefes alıyor" gibi olur. Yaylı/korozyonlu metalde çok etkili.

### 3.5 `size` ve `material` parametreleri

**Tek `size` knob'u → tüm tablo.** Oranlar sabit, sadece `f0` ve `T60` kayar:

```
f0(size)   = f0_ref · (size_ref / size)^p        p = 1.5   (bending 2.0 ile shell 1.0 arası)
f_k        = ratio_k · f0
T60_k      = T60_0 · (size/size_ref)^0.35 · ratio_k^(−α)
T60_0(size)= T60_0_ref · (size/size_ref)^0.35
```

`p = 1.5`, `size ∈ [0.5, 2.0]` → `f0` ×2.83 … ×0.35 (1.5 oktav aşağı, 1.5 oktav yukarı).
`T60` üssü 0.35: büyük nesne uzun çınlar (daha düşük `f` → daha az radyasyon + daha büyük kütle).

Doğrulama: çelik boru `L` 0.5 → 1.0 m: `f1` 762 → 190 Hz (bending, p=2). `p=1.5` ile 269 Hz. Gerçek bir
varil 1 m'de ~200–300 Hz'de çalar → `p = 1.5` doğru seçim.

**`material` morph — indeks eşlemesi yanlış yol.** Kemik 10 mod, ahşap 9 mod; `k`-ıncı modu `k`-ıncı modla
interpole edersen morph sırasında modlar "ziplar". Doğrusu **sabit log-frekans ızgarasına yeniden örnekleme**:

```rust
/// İki tabloyu N slotta birleştir. Slot oranları sabit (geometrik).
pub fn morph(a: &Material, b: &Material, t: f32, n: usize) -> Vec<Mode> {
    let r_max = 16_000.0 / a.f0_hz.min(b.f0_hz);   // en tiz slot
    (0..n).map(|i| {
        // slot oranı: 1 .. r_max geometrik
        let r = r_max.powf(i as f32 / (n - 1) as f32);
        let (fa, aa, ta) = sample_table(a, r);     // ln(f) üzerinde lineer interp
        let (fb, ab, tb) = sample_table(b, r);
        let f = (fa.ln() * (1.0 - t) + fb.ln() * t).exp();
        let g = (aa.max(1e-4).ln() * (1.0 - t) + ab.max(1e-4).ln() * t).exp();
        let d = (ta.ln() * (1.0 - t) + tb.ln() * t).exp();
        Mode { freq: f, amp: g, decay: d }
    }).collect()
}
```

`sample_table(mat, r)`: tabloda `r`'yi çevreleyen iki modu bul, `ln(amp)` ve `ln(T60)`'ı `ln(f)` üzerinde
lineer interpole et; tablo dışındaysa en yakın uç modun değerini `r^(-α)` ile ekstrapole et, `amp → 0`.
Hepsi `prepare()` zamanında, ses thread'inde değil. Sonuç: 12 slot her zaman dolu, morph sürekli.

### 3.6 Somut API

Mevcut `Mode { freq, amp, decay }` **korunur** (52 çağrı yeri var). Eklenenler:

```rust
// ---- sg-dsp/src/modal.rs ----

/// Tablo satırı: oran + genlik + ikizleşme. Frekans/T60 buradan türetilir.
#[derive(Clone, Copy, Debug)]
pub struct MatMode {
    /// f_k / f_0
    pub ratio: f32,
    /// Lineer genlik, tablo içi max = 1.0 (pickup comb henüz uygulanmadı).
    pub amp: f32,
    /// Doublet ayrışması: 0.0 = tekli, 0.01 = %1. Bu satır iki resonator'a açılır.
    pub split: f32,
}

/// Bir malzemenin tam tanımı. `static` olarak yaşar, allocation yok.
pub struct Material {
    pub id: &'static str,
    /// Referans boyuttaki temel frekans (Hz).
    pub f0_hz: f32,
    /// Referans boyuttaki T60 (s).
    pub t60_0: f32,
    /// Sönüm yasası üssü: T60_k = t60_0 * ratio^-alpha.
    pub alpha: f32,
    pub modes: &'static [MatMode],
    /// Sesin ne kadarı gürültü: 0 = tamamen modal, 1 = tamamen noise.
    pub noise_mix: f32,
    /// Önerilen kontak süresi (s) — bkz. §3.2 tablosu.
    pub contact: f32,
}

/// Vurma / dinleme noktaları.
#[derive(Clone, Copy, Debug)]
pub struct Pickup {
    pub strike: f32,   // 0..1
    pub listen: f32,   // 0..1
    pub depth: f32,    // 0..1, 0 = comb yok (cartoon / et)
}

/// Tek seslik, allocation-free modal gövde.
pub struct ModalBody {
    coef: [Coef; MAX_MODES],     // a1, a2 (4 set: E = 0/.33/.66/1), g, y1, y2, lam
    n: usize,
    pickup: Pickup,
    mu: f32,                     // nonlinearity
    sr: f32,
}

impl ModalBody {
    /// Tablodan katsayıları kur. Allocation yok, `MAX_MODES` sabit.
    pub fn new(mat: &'static Material, sr: f32) -> Self;

    /// Boyut / sönüm / parlaklık / dinleme noktası. `prepare` benzeri; ses thread'inde
    /// çağrılabilir çünkü yalnızca katsayı yazar (3 exp + 2 cos × n, ~200 cycle).
    pub fn configure(
        &mut self,
        size: f32,            // 0.5..2.0, f0 = f0_ref * (1/size)^1.5
        damping_scale: f32,   // 0.25..4.0, T60 çarpanı
        brightness: f32,      // 0..1, amp *= ratio^(-1.2 + 1.6*brightness)
        pickup: Pickup,
    );

    /// Vuruş: modları başlangıç koşuluyla başlatır (§3.2-A). `vel` 0..1.
    /// `style` 0..1 → cartoon morph (§4). NaN/Inf üretmez, `vel` clamp'lenir.
    pub fn strike(&mut self, vel: f32, style: f32, rng: &mut Rng);

    /// Sürekli uyarım (sürtme, kayma). Vuruşla aynı bankı kullanır.
    #[inline]
    pub fn excite(&mut self, force: f32);

    /// Blok render. Çıkış `out`'a **eklenir** (mevcut `Layer` sözleşmesi).
    pub fn process(&mut self, out: &mut [f32]);

    /// Kalan enerji (tail trimming için). `modal.rs:57`'deki gibi her örnek çağırma.
    pub fn energy(&self) -> f32;

    /// İki malzeme arasında morph edilmiş tablo (§3.5). Preset kurulumunda çağrılır.
    pub fn blended(mat_a: &Material, mat_b: &Material, t: f32, n: usize) -> Vec<Mode>;
}

/// Mevcut `ModalBank` ile uyumluluk: tabloyu düz `Vec<Mode>`'a çevir.
/// Bu, render.rs'i değiştirmeden yeni tabloları devreye almanın 1 saatlik yolu.
pub fn table_to_modes(
    mat: &Material,
    size: f32,
    contact: f32,
    pickup: Pickup,
    rng: &mut Rng,
) -> Vec<Mode>;
```

**Migrasyon sırası (en çok kazanç / en az risk önce):**

1. `table_to_modes` + §2 tabloları → `weapons.rs`, `gore.rs`, `zombies.rs`, `crates.rs` içindeki elle yazılmış
   `metal_modes`/`wood_modes`/`PLATE` fonksiyonlarını değiştir. `render.rs` **hiç değişmez**.
2. `ModalBank`'a IR-enerji normalizasyonu (§3.2) → `gain_db` elle trim'i biter.
3. `LayerKind::Modal`'a `velocity: f32` ve `pickup: (f32, f32, f32)` alanları.
4. `ModalBody`'ye geçiş + `strike()` (initial condition) + nonlinearity.
5. Cartoon morph'u `style` üzerinden `strike()`'a bağla.

---

## 4. Gerçekçi ↔ Cartoon Morph

### 4.1 Hangi parametre gerçekçilik taşıyor (algısal sıralama)

| Sıra | Parametre | Neden | Gerçek | Cartoon |
|---|---|---|---|---|
| **1** | **Sönüm yasası `α`** | α=0 "org" demek. Kulak bunu 50 ms'de ayırt eder. | 0.42–1.0 | **0** (tüm modlar aynı T60) |
| **2** | **Pickup comb `depth`** | Spektrumdaki çukurlar "vurulmuş cisim" imzası | 0.7–0.9 | **0.05–0.15** |
| **3** | **Splitting `δ`** | Beat = "canlı nesne" | %0.3–2 | **0** |
| **4** | **Oran seti (inharmonicity)** | Harmonik = müzikal; oran seti kimlik | fiziksel | `→ k` (harmonik) |
| **5** | **Mode sayısı** | 8 vs 24 farkı, comb+damping doğruysa **küçük** | 9–12 | **3–5** |
| **6** | **Amp tilt (parlaklık)** | 4 kHz üstü enerji "sertlik" | `ratio^-1.2` | `ratio^-0.3` ama 8 kHz LP |
| **7** | **Mistune** | %1.5 "gerçek", %0 "temiz" | %0.8–2 | **%0** |

**Sürpriz ama doğru:** mistune cartoon'da **azalır**. Oyuncak/çizgi film nesnesi akortludur; gerçek nesne
dağınıktır. Mevcut kodun `detune_cents + style*35.0` (`render.rs:191`) ters yönde — cartoon nesneyi
*akortsuzlaştırıyor*. Bu, "bozuk synth" hissinin bir kaynağı.

### 4.2 Somut morph eşlemesi

```
p(style) = p_real + (p_cartoon − p_real) · curve(style)
```

| Parametre | curve | Not |
|---|---|---|
| `T60` çarpanı | `linear` | `×1.0 → ×2.2` |
| `α` | `linear` | `α_real → 0.0` (α=0 en güçlü cartoon sinyali) |
| `pickup.depth` | `smoothstep` | `d → 0.08` |
| `split δ` | `linear` | `δ → 0` (style 0.3'te biter) |
| `ratio_k` | `linear` | `lerp(ratio_k, k, 0.55·style)` |
| `mistune` | `linear` | `%1.5 → %0` |
| `mode_count` | `stepped(3)` | `12 → 4`, eşikler 0.33 / 0.66 |
| `brightness` | `linear` | `0 → 0.75`, ama `LP 8 kHz` ekle |
| `contact` | `linear` | `T_c → 3 × T_c` (daha yumuşak, "squash") |
| `nonlinearity μ` | `linear` | `μ → 0` (cartoon lineer) |

### 4.3 Malzeme bazlı `style=1` değerleri

| Malzeme | mod | α | depth | δ | T60× | oran hedefi | ek |
|---|---|---|---|---|---|---|---|
| Çelik varil | 12→5 | 0.72→0.0 | 0.70→0.05 | 0.9%→0 | 1.0→2.6 | harmonik ×0.45 | +sine 1.7 kHz "donnng" |
| Alüminyum | 10→5 | 0.85→0.0 | 0.80→0.10 | 0→0 | ×1.9 | harmonik ×0.5 | +2 kHz "tink" |
| Meşe kalas | 9→5 | 0.55→0.05 | 0.85→0.10 | 0→0 | ×1.5 | harmonik ×0.4 | +"boing" pitch env |
| Çam | 8→4 | 0.62→0.05 | 0.85→0.10 | — | ×1.4 | harmonik ×0.4 | +"tok" 400 Hz |
| Cam şişe | 9→5 | 0.42→0.0 | 0.70→0.05 | 0.6%→0 | ×2.2 | harmonik ×0.5 | +"bwong" |
| Cam kırık | 8→4 | 0.50→0.0 | 0.80→0.15 | 0→0 | ×2.8 | harmonik ×0.6 | xylophone tonu |
| Seramik | 10→5 | 0.68→0.05 | 0.80→0.10 | 0→0 | ×2.0 | harmonik ×0.5 | +"clink" 3 kHz |
| Kemik | 10→5 | 0.60→0.05 | 0.85→0.15 | 0→0 | ×2.4 | harmonik ×0.45 | +xylophone |
| Et | 7→3 | 0.50→0.20 | **0.15→0.0** | — | ×1.6 | harmonik ×0.3 | +"slap" 380 Hz sine |
| Beton | 9→4 | 0.90→0.10 | 0.80→0.10 | — | ×1.7 | harmonik ×0.5 | +"clunk" 220 Hz |
| Karton | 7→3 | 1.0→0.3 | 0.6→0.0 | — | ×2.0 | harmonik ×0.3 | +"fweep" |
| Plastik | 10→5 | 0.80→0.05 | 0.80→0.09 | 0→0 | ×2.0 | harmonik ×0.5 | +"boop" pitch drop |

**Doğrulama kuralı:** `style=1` çıktısında spektral centroid **1.4–1.9×** artmalı ve T60 **1.4–2.8×**
uzamalı. Bu ikisi tutmuyorsa morph "cartoon" değil, sadece "bozuk" olur.

**Ve bir uyarı:** mevcut `render.rs:193` `decay_scale = 1.0 + style*0.65` **tek başına** cartoon değil,
"daha büyük nesne" demek. Cartoon'un ayırt edici bileşeni `α → 0` (tüm modlar birlikte sönüyor = müzikli
çınlama) ve `depth → 0` (comb'suz, "temiz" spektrum). Bu ikisi olmadan decay uzatmak sadece kuyruğu şişirir.

---

## 5. Mevcut Kodla Fark Analizi

### 5.1 `crates/sg-dsp/src/modal.rs`

| Yer | Ne var | Ne olmalı | Etki |
|---|---|---|---|
| `modal.rs:8-15` `Mode` | `freq, amp, decay` (mutlak Hz) | `MatMode { ratio, amp, split }` + `Material` | Şu an hiçbir tablo yeniden boyutlanamaz; her preset Hz'i elle yazıyor (52 çağrı yeri). |
| `modal.rs:32-46` `ModeState::new` | `a1 = 2r cosθ`, `a2 = −r²` | Doğru. Ama `split` yok, `T_c` yok, pickup yok | — |
| `modal.rs:36` | `r` klamp `0.99999` | Doğru | — |
| `modal.rs:49-54` `process` | `y = a1y1 + a2y2 + x; y*amp` | Aynı + 4 katsayı seti arası interp (E'ye göre), nonlinearity enjeksiyonu | Genliğe bağlı sönüm **yok** → sert/yumuşak vuruş farksız. |
| `modal.rs:53` `y * self.amp` | `amp` çıkışta | Aynı, ama `prepare`'da IR-enerji normalizasyonu şart | 5 modlu tablo 12 modludan **−4.6 dB** farklı. |
| `modal.rs:57-58, 90-96` `energy`/`is_silent` | Var | **Ölü kod** — hiçbir yerde çağrılmıyor (`grep is_silent` = 0 çağrı) | Kuyruk kesme kullanılmıyor; her modal katman recipe süresini baştan sona render ediyor. |
| `modal.rs:102-115` `inharmonic_modes` | `f = base · n^stretch` | Fiziksel seri (§1.1/1.2/1.3) | **Ölü kod** — sadece `sg-engine/src/lib.rs:17`'de re-export; hiçbir preset çağırmıyor. `stretch=1.0` tam harmonik üretir: mümkün olan en yapay set. |
| `modal.rs:111` | `d = decay/(1+n*0.35)` | `T60_0 · ratio^-α`, `α` malzemeden | Etkin `α ≈ 0.31` — gerçek 0.5–1.0. **Tiz modlar 2–3× fazla uzun.** En çok duyulan "synthetic" sebebi. |
| `modal.rs:62-71` `ModalBank` | `Vec<ModeState>` — **AoS** | `[f32; MAX_MODES]` × 8 dizi — **SoA** | AVX2 8-lane kaybı; 3–4× yavaş. |
| (yok) | — | Pickup comb (§3.3) | **En büyük tek kazanç.** |

### 5.2 `crates/sg-engine/src/render.rs`

| Yer | Ne var | Sorun |
|---|---|---|
| `render.rs:183-222` `LayerKind::Modal` | Tek `exc` üretip `bank.process(exc)` | Aynı uyarım **bütün** modlara giriyor. Vurma noktasında düğüm olan mod bile tam enerji alıyor — fiziksel olarak imkânsız. |
| `render.rs:191` | `detune_cents + style*35.0` | Cartoon nesneyi 35 cent tiz + **akortsuz** yapıyor. Doğrusu: cartoon akortlu olur (§4.1). |
| `render.rs:193` | `decay_scale = 1 + style*0.65` | Uniform çarpan → `α` değişmiyor → "büyük nesne", "cartoon" değil. |
| `render.rs:208` | `exc_len = exciter_ms * sr` | `exciter_ms = 10` (`crates.rs:131`) → 10 ms temas = 100 Hz LPF. Vuruş değil, sürtme. |
| `render.rs:211-218` | `i == 0` ise `exc = 1.0` | Tek örnek impuls = tüm frekanslarda eşit fazlı enerji → metalik "tık". `T_c` (§3.2) yok. |
| (yok) | — | **`velocity` kavramı yok.** `LayerKind::Modal` (`layer.rs:117-126`) içinde alan yok → genliğe bağlı sönüm ve nonlinearity bağlanamaz. |
| (yok) | — | Normalizasyon yok: `out[i] += bank.process(exc)` ham toplam. |

### 5.3 Presetler

| Yer | Sorun | Düzeltme |
|---|---|---|
| `weapons.rs:32-48` `metal_modes` | 5 mod, doublet yok, `decay*(1-i*0.14)` → `α_eff ≈ 0.26` | §2.1 tablosu (`table_to_modes`) |
| `weapons.rs:50-61` `wood_modes` | 5 mod, `decay*(1-a*0.25)` — genliğe bağlı, fizik dışı | §2.3 |
| `gore.rs:13-15` | `PLATE = [1, 1.594, 2.136, 2.296]` = **membran**, plaka değil | §2.7 kemik tablosu |
| `gore.rs:19-31` `struck` | `amp = 1/(1+0.9i)`, `d = decay*(1-0.12i).max(0.4)` | `α` yok, `ratio` yok |
| `zombies.rs:26-36` `bone_modes` | 4 mod; membran serisinin ilk 4'ü → 1.6/2.1/2.3 kHz kümesi baskın | §2.7 |
| `crates.rs:131` | `exciter_ms = 10.0`, pink, 220–3200 Hz | `0.6 ms` + beyaz; `T_c` modeli |
| `crates.rs:431, 446` | `exciter_ms = 0.0` "single-sample impulse" | `T_c` + pickup ile değiştir |
| Tüm presetler | `Mode::new(abs_hz, ...)` — boyut bilgisi yok | `Material` + `size` |
| Hiçbir preset | 10 kHz üstü mod yok | Gerçek impact'te 12–16 kHz var. §2 tabloları oraya uzanıyor. |

### 5.4 Öncelik sırası (etki / efor)

| # | İş | Efor | Beklenen kazanç |
|---|---|---|---|
| 1 | `table_to_modes` + §2 tabloları, presetleri değiştir | ~3 sa | **Çok yüksek** — oran setleri + 12 mod + doğru `α` |
| 2 | Pickup comb (§3.3) `struck()` içine | ~1 sa | **Çok yüksek** — "vurulmuş nesne" imzası |
| 3 | `ModalBank` IR-enerji normalizasyonu | ~30 dk | Yüksek — kütüphane loudness tutarlılığı |
| 4 | Doublet'ler (`split` alanı) | ~2 sa | Yüksek — beat = canlılık |
| 5 | `velocity` + genliğe bağlı sönüm (§1.7) | ~4 sa | Yüksek — vuruş şiddeti ayırt edilir |
| 6 | SoA'ya geçiş | ~3 sa | Orta — CPU 0.06% → 0.012%; şimdilik gereksiz |
| 7 | Nonlinearity (§3.4) | ~2 sa | Orta-yüksek — sadece sert vuruşlarda duyulur |
| 8 | `ModalBody` + `strike()` IC | ~1 gün | Yüksek ama 1–5'in üstüne biner |
| 9 | Material morph (§3.5) | ~3 sa | Orta — UI/keşif değeri |

**Tek bir değişiklik seçmek zorunda olsaydın:** pickup comb (#2). İkinci: `α` (#1).

---

## 6. Kopyala-yapıştır: Rust tabloları

`MatMode` satırları (`ratio, amp` — `split` set edilmemişse 0; bu tablolar doublet'leri **hazır** içerir):

```rust
// §2.1 steel barrel  f0=480  t60=1.90  alpha=0.72  contact=0.00015
r: [1.000,1.009,2.756,2.790,5.404,5.470,8.933,9.062,13.340,17.710,23.530,30.100]
a: [1.00, 0.50, 0.62, 0.34, 0.40, 0.22, 0.25, 0.14, 0.16, 0.10, 0.07, 0.04]

// §2.2 aluminium sheet  f0=1180  t60=0.34  alpha=0.85  contact=0.00020
r: [1.000,1.729,2.328,3.906,4.044,6.137,6.688,8.612,10.900,13.200]
a: [0.72, 0.44, 1.00, 0.30, 0.18, 0.20, 0.12, 0.09, 0.06, 0.04]

// §2.3 oak plank  f0=158  t60=0.26  alpha=0.55  contact=0.00080
r: [1.000,1.630,2.712,5.240,8.550,12.620,17.200,22.300,28.300]
a: [0.85, 0.55, 1.00, 0.42, 0.24, 0.14, 0.08, 0.05, 0.03]

// §2.4 pine board  f0=124  t60=0.185  alpha=0.62  contact=0.00080
r: [1.000,1.580,2.690,5.200,8.440,12.400,16.900,21.900]
a: [0.80, 0.45, 1.00, 0.36, 0.18, 0.10, 0.05, 0.03]

// §2.5a glass bottle  f0=520  t60=1.90  alpha=0.42  contact=0.00020
r: [0.420,1.000,1.006,2.830,2.860,5.420,8.770,12.900,17.700]
a: [0.30, 1.00, 0.62, 0.55, 0.35, 0.28, 0.18, 0.10, 0.06]
// not: ratio 0.420 satırı Helmholtz — T60'ı yasadan değil, elle 0.22 s ver.

// §2.5b glass shard  f0=1800  t60=0.095  alpha=0.50  contact=0.00015
r: [1.000,1.729,2.328,3.906,4.044,6.137,6.688,8.612]
a: [0.75, 1.00, 0.85, 0.42, 0.30, 0.22, 0.14, 0.09]

// §2.6 ceramic tile  f0=1080  t60=0.235  alpha=0.68  contact=0.00020
r: [1.000,1.660,2.280,3.720,4.150,6.020,6.550,8.400,10.500,12.800]
a: [0.85, 1.00, 0.78, 0.40, 0.32, 0.20, 0.13, 0.09, 0.05, 0.03]

// §2.7 bone femur  f0=860  t60=0.105  alpha=0.60  contact=0.00025
r: [1.000,1.620,2.720,2.780,3.940,5.310,5.440,7.900,9.200,11.600]
a: [0.80, 1.00, 0.55, 0.35, 0.45, 0.28, 0.18, 0.12, 0.07, 0.04]

// §2.8 flesh  f0=92  t60=0.048  alpha=0.50  contact=0.004  noise_mix=0.70
r: [1.000,1.850,3.200,5.100,7.400,10.500,14.200]
a: [1.00, 0.55, 0.30, 0.16, 0.09, 0.05, 0.03]

// §2.9 concrete  f0=155  t60=0.38  alpha=0.90  contact=0.00060  noise_mix=0.45
r: [1.000,1.520,2.310,3.140,4.420,6.050,8.100,10.600,13.800]
a: [1.00, 0.62, 0.48, 0.30, 0.22, 0.14, 0.09, 0.05, 0.03]

// §2.10 cardboard  f0=430  t60=0.013  alpha=1.00  contact=0.003  noise_mix=0.60
r: [1.000,1.900,3.300,5.200,7.600,10.400,14.000]
a: [1.00, 0.45, 0.28, 0.16, 0.09, 0.05, 0.03]

// §2.11 plastic shell  f0=720  t60=0.125  alpha=0.80  contact=0.00040
r: [1.000,1.720,2.350,3.880,4.100,6.100,6.720,8.500,10.800,13.500]
a: [0.85, 1.00, 0.70, 0.35, 0.25, 0.16, 0.10, 0.07, 0.04, 0.03]
```

---

## 7. Test / doğrulama (implementasyon sonrası zorunlu)

| Test | Yöntem | Geçme kriteri |
|---|---|---|
| IR enerjisi | Render edilen impulse cevabının `Σy²` | Tüm malzemelerde ±0.5 dB içinde |
| Sönüm yasası | Her modun zarfından T60 ölç, `log T60` – `log f` eğimi | Eğim `−α ± 0.08` |
| Pickup comb | FFT'te çukur derinliği | `depth 0.85` → en az 3 çukur ≥ −12 dB |
| Beat | 480 Hz doublet'in zarfında | 3–6 Hz modülasyon, derinlik 4–10 dB |
| Kararlılık | `mu = 0.45`, `vel = 1.0`, 10 s render | `|y| < 4.0`, NaN yok |
| Katsayı interp | 4 set arası rastgele `E` ile 10⁵ örnek | `|r_dyn| < 1` her zaman |
| Determinizm | Aynı seed, 2 render | Bit-identical |
| CPU | 24 voice × 12 mod, 60 s | < 2% tek çekirdek |
