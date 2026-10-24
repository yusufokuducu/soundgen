# 03 — Yaratık Sesi: Source-Filter Vokal Sentezi

> **Kapsam:** İnsan-dışı yaratık vokalleri için glottal source + formant tract modeli.
> Mevcut `zombies.rs` presetlerinin neden "synthetic/electronic" duyulduğunun ölçümlü teşhisi,
> üç glottal pulse modelinin tam zaman-domeni formülleri, non-human modifikasyonlar (subharmonic,
> jitter/shimmer, roughness, false-fold), 7 presetlik yaratık formant tablosu, gurgle ve breath
> katmanlarının somut parametreleri, ve `VoiceSource` + `FormantBank` API önerisi.
> **Hedef okuyucu:** yarın `sg-dsp/src/voice.rs` yazacak geliştirici.
> Tüm değerler **48 kHz** içindir. `§0`'daki tablolar bu repoda fiilen ölçülmüştür.

---

## 0. Teşhis: Mevcut Çıktı Neden Elektronik?

`sg-presets` zombie presetleri render edilip 8 oktav bandında (Svf band-pass, Q=1.4, 315 Hz'e göre
normalize) ve autocorrelation periodicity ile ölçüldü. Sonuç:

| preset | 80 | 160 | 315 | 630 | 1250 | 2500 | 5000 | 10k | periodicity |
|---|---|---|---|---|---|---|---|---|---|
| zombie_growl_low | **+3.3** | −0.5 | 0.0 | −1.2 | −5.3 | −9.2 | −14.3 | −19.8 | **0.28** |
| zombie_growl_high | +1.4 | +3.8 | 0.0 | −1.0 | −2.1 | −3.6 | −8.1 | −14.0 | **0.41** |
| zombie_scream | −14.0 | −7.2 | 0.0 | +3.2 | −0.4 | −1.8 | −6.0 | −11.6 | **0.21** |
| zombie_pain_hit | −0.6 | +3.5 | 0.0 | −1.7 | −5.1 | −8.4 | −12.6 | −17.5 | 0.37 |
| zombie_attack_lunge | +3.0 | +2.0 | 0.0 | −1.1 | −2.4 | −4.7 | −8.9 | −14.4 | 0.50 |
| zombie_bite | −3.3 | +3.7 | 0.0 | +3.0 | +3.3 | +0.1 | −4.6 | −11.9 | 0.40 |
| zombie_death_rattle | +3.1 | +0.4 | 0.0 | −2.2 | −5.9 | −10.0 | −14.4 | −19.4 | **0.27** |
| zombie_idle_breath | 0.0 | −3.0 | 0.0 | +2.0 | +1.6 | −1.6 | −7.5 | −14.9 | 0.33 |
| zombie_horde_bed | +3.4 | +0.4 | 0.0 | −2.4 | −6.0 | −9.7 | −13.8 | −18.7 | 0.39 |

`(dB, 315 Hz = 0; periodicity = normalize edilmiş autocorrelation tepe değeri, 1.0 = tam periyodik)`

Üç ölümcül bulgu:

1. **Hiçbir preset'te formant yok.** Spektrum monoton bir eğri: 315 Hz'ten yukarısı düzgün düşüyor,
   hiçbir lokal maksimum yok. Gerçek bir growl'da F1 (400–700 Hz) komşu oktavlara göre **+8…+15 dB**,
   F2 (800–1200) **+5…+8 dB**, F3 (2.1–2.5 kHz) **+3…+6 dB** tepe yapar. Master chain'deki
   `presence_db 2.5 @1700 Hz` bunu taklit edemez: 2.5 dB'lik bir shelf, 1/3-oktav pencere içinde
   *duyulabilir bir rezonans* için gereken ≥6 dB eşiğinin altında.
2. **Periodicity 0.21–0.50.** Sesli (voiced) bir kaynağın baskın olduğu bir sinyalde bu değer
   0.80–0.95 olur (§2.2'deki ölçüm tablosu). Düşük değer, duyulan şeyin *periyodik kaynak* değil,
   *gürültü katmanları* olduğunu söylüyor.
3. **Sebep katman gain'lerinde.** `growl_low`: voice −7 dB, "throat rasp" **+6 dB**, creak **+4 dB**,
   breath **0 dB**. `horde_bed`: üç voice −10/−11/−12 dB, "rasp bed" **+1 dB**. Yani her preset'te
   en az bir gürültü katmanı vokalden 5–17 dB daha yüksek. Kulağa gelen "hiss + buzz" tam olarak bu.

Ek olarak, kaynak tarafında üç ayrı "synth" işareti var:

| işaret | nerede | neden elektronik |
|---|---|---|
| `WaveShape::Saw` @ 61–180 Hz | tüm voice katmanları | sonsuz harmonik serisi, −6 dB/oct, sıfır yükselme süresi (instantaneous reset), formant zarfı yok |
| `fm_hz 1.01–1.04`, `fm_depth 0.15–0.25` | tüm `Tone` katmanları | modülatör = 1.02×f0 ⇒ ±2 % detune. Bu **supersaw chorus**'tur, jitter değil |
| `vibrato` 5.5 Hz sinüs | tüm `Tone` katmanları | ±4–6 % = ±70…±100 cent, *tam periyodik*. En güçlü "şarkıcı/synth" ipucu |

Ve `master_chain`'deki `Saturator` 1× (oversampling yok): 4–6 dB drive, Nyquist'e kadar harmonik
içeren bir saw'da intermodülasyon alias'ı üretir (−35…−45 dB bandında "dijital fizz").

**Sonuç:** sorun preset ayarları değil, motorun ifade gücü. `LayerKind::Tone` bir formant bankına,
periyodik olmayan jitter'a veya ikinci bir kaynağa sahip değil. §6'daki API bu boşluğu kapatıyor.

---

## 1. GLOTTAL SOURCE MODELLERİ

### 1.1 Önce spektrum: hangi eğri "boğaz", hangisi "buzzer"

Fizik: gırtlaktan çıkan **hacim hızı** `U(t)` vocal tract'i sürer; mikrofona gelen basınç
`p(t) ∝ dU/dt` (lip radiation = bir fark alıcı, +6 dB/oct). Yani:

| model | alçak bölge eğimi | kırılma | yüksek bölge | spektral sıfır |
|---|---|---|---|---|
| ideal saw (osilatör) | −6 dB/oct | — | −6 dB/oct | — |
| glottal **flow** (Rosenberg `g`) | −12 dB/oct | ≈ `1/(2π·T2)` | −18…−24 dB/oct | çıkış lobu nedeniyle |
| flow **derivative** (LF `E`) = radiated source | −6 dB/oct | `f_a = 1/(2π·Ta)` | −12 dB/oct | `f_z ≈ 3–4·f0` |
| saw + drive + EQ | −6 dB/oct | keyfî | −6…−3 dB/oct | — |

Bu repoda 8192-nokta DFT ile ölçtüğüm gerçek harmonik zarf (f0 = 80 Hz, dB):

| model | h1 | h2 | h3 | h4 | h5 | h6 | h8 | eğim h2–h8 | eğim h8–h25 |
|---|---|---|---|---|---|---|---|---|---|
| saw | −9.9 | −16.0 | −19.5 | −22.0 | −23.9 | −25.5 | −28.0 | **−6.0** | −6.0 |
| Rosenberg flow türevi (T1=.40, T2=.16) | −16.4 | −17.3 | −23.9 | −24.0 | −28.1 | −29.9 | −34.8 | **−8.8** | −5.5 |
| LF türevi (Rg=1.43, Te/T0=.60, Ta/T0=.02) | −35.1 | −31.0 | −31.8 | −33.0 | −34.0 | −34.9 | −37.0 | **−4.7** | −5.8 |

Üç kritik okuma:

- **h1/h2 ilişkisi.** LF'te h1, h2'den **4 dB düşük**. Saw'da h1 en yüksek. Bu, negatif "return
  lobe"un alçak harmonikleri götürmesidir: kaynak kendi içinde bir high-pass gibi davranır.
  "Booming synth" hissi tam olarak bu fazlalıktan gelir ve hiçbir statik EQ bunu geri alamaz
  (çünkü sıfırın yeri f0 ile birlikte kayar).
- **Kapanış dikliği.** Rosenberg'de T2, h2–h8 aralığına fazladan **2.8 dB/oct** ekler. T2'yi
  büyütmek (0.10 → 0.25 T0) kaynağı yumuşatır = "nefesli", küçültmek (0.10) sertleştirir = "gıcık".
- **Formantın önündeki fark.** f0 = 80 Hz'de 2 kHz = h25. Saw −28.0 dB, flow −55.9 dB:
  **28 dB**. İşte "buzzer vs boğaz" farkının sayısı bu. 4 kHz'de (h50) fark 34 dB.

### 1.2 Rosenberg pulse (1971) — varsayılan model

Normalize faz `u = t/T0 ∈ [0,1)`:

```
g(u) = 0.5 · (1 − cos(π·u/T1))              0      ≤ u < T1
g(u) = cos(π·(u−T1)/(2·T2))                 T1     ≤ u < T1+T2
g(u) = 0                                    T1+T2  ≤ u < 1
```

`g` sürekli ve `C¹`'dir (her iki eklemde türev 0), tepe değeri 1'dir. Türevde kapanış anında
sıçrama yoktur — bu yüzden Rosenberg tek başına "yumuşak" duyulur; sertliği **return phase** verir.

| parametre | aralık | varsayılan | etki |
|---|---|---|---|
| `T1/T0` (open quotient) | 0.30–0.55 | 0.40 | büyük = gevşek/nefesli; 0.3 = pressed/creak |
| `T2/T0` (speed quotient) | 0.08–0.25 | 0.16 | küçük = hızlı kapanış = daha parlak, daha "gıcık" |
| `Ta/T0` (return phase) | 0.005–0.060 | 0.020 | negatif lobun süresi; `f_a = 1/(2πTa)` low-pass |
| açık faz `T1+T2` | 0.40–0.80 | 0.56 | >0.80 = "kaçak/nefesli", <0.40 = "gırtlaktan kopuk" |

Return phase (kapanış sonrası negatif lob), `g`'yi bir kez türevledikten sonra eklenir:

```
h(u) = g(u) − g(u−1)                              // flow derivative
h(u) −= Aret · exp(−(u − T1 − T2)/Ta)             // u > T1+T2, Aret = 0.15…0.40
```

`Aret` negatif lobun yüksekliğidir: 0.15 = yumuşak, 0.40 = sert kapanış (sert growl). Negatif
lobun alanı pozitif lobun ~%10–20'si kadar olmalı; daha büyüğü kaynağı fazla "tizleştirir".

### 1.3 LF model (Fant/Liljencrants/Lin 1985) — yüksek kalite

`E(t)` = glottal flow türevi (yani doğrudan tract'e giren excitation):

```
E(t) = E0 · e^(α·t) · sin(ω_g·t)                                    0 ≤ t ≤ Te
E(t) = −(Ee/(ε·Ta)) · [ e^(−ε·(t−Te)) − e^(−ε·(Tc−Te)) ]            Te < t ≤ Tc
```

`Tc = T0`. Süreklilik ve tepe koşulları:

```
E'(Te) = 0        ⇒   α = −ω_g · cot(ω_g·Te)
E(Te)  = −Ee      ⇒   E0 normalize edilir (tablo kurulurken max |E| = 1)
ε·Ta   ≈ 1        ⇒   ε = 1/Ta
```

**Kritik kısıt (bunu atlayan implementasyonlar bozuk çıkar):** `α > 0` ve `E(Te) < 0` aynı anda
ancak `ω_g·Te ∈ (3π/2, 2π)` iken sağlanır. `Rg ≡ ω_g/ω0` ve `s ≡ Te/T0` ile bu,
**`0.75 < Rg·s < 1.0`** demektir. Dolayısıyla `Rg` serbest değildir: `s` seçilince
`Rg ≈ 0.86/s`. Bu bölgede çözümler (hesaplanmış):

| `s` = Te/T0 | `Rg` = ω_g/ω0 | `ω_g·Te` | `α·T0` | `Tp/T0` (tepe) | `f_z/f0` |
|---|---|---|---|---|---|
| 0.45 | 1.91 | 310° | 9.93 | 0.188 | 3.82 |
| 0.50 | 1.72 | 310° | 8.94 | 0.209 | 3.44 |
| 0.55 | 1.56 | 310° | 8.13 | 0.230 | 3.13 |
| 0.60 | 1.43 | 310° | 7.45 | 0.251 | 2.87 |
| 0.65 | 1.32 | 310° | 6.88 | 0.272 | 2.65 |
| 0.70 | 1.23 | 310° | 6.39 | 0.293 | 2.46 |
| 0.75 | 1.15 | 310° | 5.96 | 0.314 | 2.29 |

`f_z = 1/(Te − Tp)`: açılış ve kapanış loblarının girişimiyle oluşan spektral sıfır.
**3–4·f0**'da oturur, yani pitch ile birlikte kayar — saw'da böyle bir şey yoktur.

`Ta/T0` → return-phase köşesi (f0 = 80 Hz için):

| `Ta/T0` | `Ta` (ms) | `f_a = 1/(2πTa)` | karakter |
|---|---|---|---|
| 0.005 | 0.06 | 2546 Hz | pressed, parlak, "gıcık" |
| 0.010 | 0.13 | 1273 Hz | modal |
| 0.020 | 0.25 | 637 Hz | hafif nefesli |
| 0.040 | 0.50 | 318 Hz | breathy, "yorgun" |
| 0.060 | 0.75 | 212 Hz | fısıltıya yakın |

**Uygulama notu:** `α·T0` 6–10 aralığında `e^(α·t)` f32'de taşar. Tabloyu `f64` ile kur ve
`max|E|`'ye böl; per-sample çalışma sırasında asla `exp` çağırma (tablo).

### 1.4 Polynomial (kübik) glottal flow — "sigh" ve aspiration için

En ucuz model, `C¹` ve kapalı-form:

```
u(x) = 3x² − 2x³,        x = t/T1,   0 ≤ x ≤ 1     // 0→1, u'(0)=u'(1)=0
u(x) = 1 − (3y² − 2y³),  y = (t−T1)/T2,  T1 ≤ t ≤ T1+T2
u = 0                                              // sonrası
```

Türevi `6x(1−x)/T1` = parabolik bir "bump": **closure spike yok** (sürekli türev). Bu yüzden:
sert growl için değil, **aspiration kaynağı** ve "son nefes / sigh" için idealdir. 4 satır kod,
sıfır tablo, sıfır `cos`. Growl'da kullanılırsa "rüzgâr gibi" duyulur — istenen de bu olabilir
(`death_rattle`'ın son nefesi).

### 1.5 Implementasyon: tablo mu, waveshaper mı

| yöntem | maliyet/örnek | kalite | ne zaman |
|---|---|---|---|
| phase accumulator + `cos` (Rosenberg) | 1 `cos` + ~6 FLOP | mükemmel | offline, tek voice |
| **tablo 2048 + linear interp** | ~5 FLOP, 8 KB | yeterli | **varsayılan** |
| tablo 4096 + Hermite | ~12 FLOP, 16 KB | çok iyi | `f0 > 250 Hz` (scream) |
| polynomial waveshaper | ~8 FLOP, 0 KB | orta | subharmonic/creak, 2. kaynak |

Aliasing analizi (tablo 2048, f0 = 72 Hz): okuma artışı `f0/sr·2048 = 3.07` örnek/çıktı-örneği;
tablo Nyquist'i `2048·72/2 = 73.7 kHz`. 24 kHz üstü ilk harmonik h≈333, seviyesi −55 dB → **pratikte
alias yok**, band-limiting gerekmez; kaynağın kendi rolloff'u işi bitiriyor. `f0 = 560 Hz`'de
artış 23.9'a çıkar ve linear interp hatası −40 dB'ye iner → scream için 4096 + Hermite kullan.

### 1.6 ~20 satırlık Rust çekirdeği

```rust
/// Rosenberg flow-derivative, single period table, linear interp.
pub struct GlottalTable {
    t: Vec<f32>,   // 2048, bir periyot, DC'siz
    sr: f32,
    phase: f32,
    f0: f32,
}

impl GlottalTable {
    pub fn new(sr: f32, t1: f32, t2: f32, a_ret: f32, ta: f32) -> Self {
        const N: usize = 2048;
        let mut g = vec![0.0f32; N];
        for i in 0..N {
            let u = i as f32 / N as f32;
            g[i] = if u < t1 {
                0.5 * (1.0 - (std::f32::consts::PI * u / t1).cos())
            } else if u < t1 + t2 {
                (std::f32::consts::PI * (u - t1) / (2.0 * t2)).cos()
            } else {
                0.0
            };
        }
        // flow derivative + return phase, circular
        let mut t = vec![0.0f32; N];
        for i in 0..N {
            let u = i as f32 / N as f32;
            let d = g[i] - g[(i + N - 1) % N];
            t[i] = if u > t1 + t2 { d - a_ret * (-(u - t1 - t2) / ta).exp() } else { d };
        }
        // remove DC (zorunlu: yoksa formant bankı offset biriktirir)
        let mean = t.iter().sum::<f32>() / N as f32;
        for v in t.iter_mut() { *v -= mean; }
        let peak = t.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-9);
        for v in t.iter_mut() { *v /= peak; }
        Self { t, sr, phase: 0.0, f0: 72.0 }
    }

    #[inline]
    pub fn tick(&mut self) -> f32 {
        let n = self.t.len();
        let x = self.phase * n as f32;
        let i = x as usize & (n - 1);
        let f = x - i as f32;
        let a = self.t[i];
        let b = self.t[(i + 1) & (n - 1)];
        self.phase += self.f0 / self.sr;          // f0 burada değişir (jitter!)
        if self.phase >= 1.0 { self.phase -= 1.0; }
        a + (b - a) * f
    }
}
```

Jitter/pitch envelope `phase` üzerinden yürür, tablo değişmez — bu yüzden model f0'dan bağımsızdır
ve bedava. **Dikkat:** `f0`'ı her örnekte güncellemek jitter'ı *duyulmaz* yapar (§2.2); periyottan
periyoda güncelle.

---

## 2. NON-HUMAN MODİFİKASYONLAR

İnsan sesini "insan" yapan şey periyodikliğin *yüksek* olmasıdır; yaratığı "yaratık" yapan şey
periyodikliğin **kontrollü biçimde bozulması**dır. Dört ayrı eksen var ve her biri farklı bir
kelimeye karşılık gelir:

| eksen | mekanizma | algı |
|---|---|---|
| subharmonic (period doubling) | periyot uzunluğunun 2/3'lü döngüsü | growl, diplophonia, "iki ses" |
| jitter | periyot süresinin stokastik sapması | pürüzlülük, "canlı", creak |
| shimmer | periyot genliğinin stokastik sapması | nefeslilik, düzensizlik |
| roughness | 30–150 Hz AM | "hırlama", gırtlak titreşimi |

### 2.1 Subharmonic / period doubling

Mekanizma: gerçek false-fold titreşimi her periyotta aynı kapanmaz. En basit model **periyot
uzunluğunu döngüsel olarak değiştirmek**tir:

```
n. periyot:  T = T0 · (1 + d · s_n)      s_n ∈ {+1, −1}        (period-2)
             s_n = +1, 0, −1  (period-3, ortalaması 0)
```

`d` = derinlik. Bu, spektrumda `f0/2` (veya `f0/3`) çizgisi üretir; genliği yaklaşık `d`'dir:

| `d` | f0/2 bileşeni | algı | kullanım |
|---|---|---|---|
| 0.00–0.02 | < −34 dB | duyulmaz, hafif "chorus" | canlılık |
| 0.02–0.05 | −34…−26 dB | hafif kalınlaşma | growl_low |
| **0.05–0.15** | **−26…−16 dB** | **growl / diplophonia** | ana growl bölgesi |
| 0.15–0.30 | −16…−10 dB | "broken", iki ayrı pitch | dying, berserk |
| > 0.30 | > −10 dB | kurbağa / ayrı vokalizasyonlar | komedi, boss |

Periyot-2 ve periyot-3 birlikte kullanılmaz (spektrum çamurlaşır). Ölen yaratıkta periyot-3,
`d = 0.10–0.20`, yavaşça artan: "ritim bozuluyor" hissi.

**Subharmonic'i f0/2 "sub-osc" ile taklit etmeyin.** Mevcut presetlerdeki `sub` katmanı tam olarak
bunu yapıyor (92 Hz'in altına saf sine 58→44 Hz). Sonuç: periyodik bir drone, yani *ikinci bir
enstrüman*. Gerçek period-doubling'de alt harmonikler **kaynağın içinden** gelir ve formant
bankından geçer; bu yüzden "boğazdan" duyulur.

### 2.2 Jitter ve shimmer

Gerçek dağılımlar (sürekli vokal / sustained vowel, yetişkin erkek):

| ölçü | normal | stresli/patolojik | yaratık hedefi |
|---|---|---|---|
| jitter (local, relative) | 0.5–1.0 % | 2–6 % | **2–4 %** growl, 6–12 % creak |
| jitter @ f0 = 80 Hz | 0.4–0.8 Hz | 1.6–4.8 Hz | 1.6–3.2 Hz |
| shimmer (local) | 3–4 % (0.25–0.35 dB) | 8–20 % | **8–15 %** (0.7–1.2 dB) |
| f0 drift (0.3–3 Hz bant) | ±1 % | ±5 % | ±3–6 % |

**Dağılım Gauss değil.** Gerçek jitter pozitif çarpık ve "sıçramalı"dır. İki bileşenli karışım
kullan (deterministik, `Rng` ile):

```
p_n = 1 + jitter · (0.85·N(0,1) + 0.15·N(0, 3.0))     // ağır kuyruk
if rng.chance(0.02) { p_n *= 1.0 + rng.range(-0.08, 0.08) }   // %2 "glottal jump"
```

**Periyot başına uygula, örnek başına değil.** Bu repoda ölçtüm: Rosenberg + growl formantları,
f0 = 72 Hz, 1.5 s:

| jitter | σ (Hz) | ölçülen periodicity | best-lag sapması |
|---|---|---|---|
| 0.0 % | 0.0 | 1.000 | 72.0 Hz |
| 0.5 % | 0.4 | 0.996 | 72.1 Hz |
| 1.0 % | 0.7 | 0.987 | 72.1 Hz |
| **2.5 %** | **1.8** | **0.937** | 72.3 Hz |
| 6.0 % | 4.3 | 0.766 | 72.5 Hz |
| 12.0 % | 8.6 | 0.515 | 72.7 Hz |

Referans: mevcut presetler 0.21–0.50. Yani 2.5 % jitter "canlı ama kontrollü", 12 % jitter
"creak/ölmek üzere" demek. Örnek-başına jitter uygulanırsa faz random-walk'a dönüşür ve
periodicity 1.000'de kalır (§1.6'daki kodun `f0`'ı her örnekte güncellemesi bu hatadır).

Shimmer aynı şekilde periyot başına: `amp = 1 + shimmer·N(0,1)`, periyot içinde sabit.

### 2.3 Roughness (30–150 Hz AM)

Roughness, kritik banttan yakın iki bileşenin baziler membranda ayrışamamasıdır; AM hızı
**50–90 Hz**'de maksimumdur, ~20 Hz altında "tremolo", ~300 Hz üstünde "ayrı olaylar" olarak
duyulur (Terhardt 1974; Zwicker & Fastl). Broadband noise'da tepe biraz daha alçaktır (~30–50 Hz).

| AM hızı | derinlik | algı |
|---|---|---|
| 4–12 Hz | 20–50 % | "nefes nefese", yorgun |
| **25–60 Hz** | **20–35 %** | **growl / hırlama** |
| 60–90 Hz | 30–50 % | maksimum roughness, "motor" |
| 90–150 Hz | 40–70 % | agresif, "testere" |
| > 150 Hz | her | buzz, pitch'e karışır |

İki kritik kural:

1. **AM hızını f0'a kilitle.** `f_am = f0/2` veya `f0/3` seçilirse modülasyon yan bantları
   harmonik ızgaraya oturur ve *subharmonic* gibi duyulur (istenen). Serbest (30–90 Hz) seçilirse
   "beating" olur — growl için de geçerli, ama daha "wah" hissi verir.
2. **Hem kaynakta hem tract'te uygula, 60/40.** Gerçek growl'da false-fold modülasyonu kaynağı
   modüle eder (formantlardan *önce*), doku/kas titremesi ise çıkışı (formantlardan *sonra*).
   Sadece post-AM "tremolo" gibi, sadece pre-AM "formant kayması olmayan buzz" gibi duyulur.

### 2.4 Sub-glottal / false-fold kaynağı

Gerçek growl'un gövdesi ikinci bir kaynaktan gelir: false vocal folds, true folds'un altında,
daha düzensiz ve daha alçak frekansta titreşir; üstü doku tarafından ~800 Hz–1 kHz'de süzülür.

| parametre | aralık | not |
|---|---|---|
| frekans | `f0/2` (kilitli) veya 25–45 Hz (serbest) | kilitli = müzikal growl, serbest = "hayvan" |
| dalga | pulse (Rosenberg, T1=0.5, T2=0.10) | sine **değil** |
| jitter | 8–20 % | true folds'tan çok daha düzensiz |
| noise karışımı | 20–35 % (pulse'a eklenen) | "ıslak hırıltı" |
| low-pass | 300–600 Hz, 2 kutup | doku süzgeci |
| band-pass | 80–250 Hz, Q 1.5 | gövde |
| gain | −10…−16 dB (vokale göre) | destek, lider değil |

Davranış: growl sırasında %100, nefes alırken %0. `attack_lunge`'da f0 ile birlikte yükselir
(120→75 Hz ise 60→38 Hz), bu "göğüsten gelen" hissini verir.

---

## 3. FORMANT MİMARİSİ

### 3.1 İnsan referansı (karşılaştırma tabanı)

| vowel | F1/BW | F2/BW | F3/BW | F4/BW | F5/BW |
|---|---|---|---|---|---|
| erkek /ɑ/ | 730/90 | 1090/110 | 2440/160 | 3300/220 | 4200/280 |
| erkek /i/ | 270/60 | 2290/100 | 3010/160 | 3500/220 | 4400/280 |
| erkek /u/ | 300/60 | 870/90 | 2240/160 | 3300/220 | 4200/280 |
| kadın /ɑ/ | 850/100 | 1220/120 | 2810/180 | 3700/240 | 4700/300 |
| çocuk /ɑ/ | 1030/120 | 1370/140 | 3170/200 | 4100/260 | 5100/320 |

Formant aralığı (spacing) `ΔF = (F4−F3 + F3−F2 + F2−F1)/3` ≈ 1000 Hz (erkek), 1200 (kadın),
1500 (çocuk). **ΔF algılanan gövde boyutudur**; F1 değeri değil. Bu yüzden yaratıklaştırmanın
ilk adımı tüm formantları ölçeklemektir.

### 3.2 Yaratık formant tabloları

Notasyon: her hücre **`F/ BW / gain_dB`** (Hz). `gain` = paralel bankın formant kazancı,
`noise` = o formanta enjekte edilen gürültü oranı (0–1). `×k` = tract ölçek çarpanı,
`hp` = kaynak high-pass (bkz. §3.4 — bu olmadan F1 görünmez).

**1. LOW GROWL** — `f0 55–90`, `×0.85`, `hp 150`, tilt −6 dB/oct

| | F1 | F2 | F3 | F4 | F5 |
|---|---|---|---|---|---|
| F/BW/g | 420/140/0.0 | 780/160/+1.0 | 2100/240/−3.0 | 2900/300/−6.0 | 3600/380/−12.0 |
| noise | 0.10 | 0.05 | 0.05 | 0.00 | 0.00 |

Ek: nasal pole 280/90 @ −10 dB; sub-glottal pole 300/120 @ −8 dB (trakea, hollow).
`ΔF ≈ 850 Hz` → ~2.1 m gövde.

**2. HIGH SNARL** — `f0 120–180`, `×0.95`, `hp 180`, tilt −6 dB/oct

| | F1 | F2 | F3 | F4 | F5 |
|---|---|---|---|---|---|
| F/BW/g | 300/110/0.0 | 1650/180/+2.0 | 2500/230/−3.0 | 3200/280/−6.0 | 4000/350/−12.0 |
| noise | 0.05 | 0.10 | 0.08 | 0.00 | 0.00 |

`F1` düşük + `F2` yüksek = "dişler açık, dil geri" = snarl. `F2−F1 = 1350 Hz` (insan /i/'de
2020) → insan değil ama ünlü de değil.

**3. SCREAM** — `f0 350–700`, `×1.05`, `hp 200`, tilt −3 dB/oct (daha az)

| | F1 | F2 | F3 | F4 | F5 |
|---|---|---|---|---|---|
| F/BW/g | 950/200/+2.0 | 1750/240/0.0 | 3050/320/−3.0 | 3900/380/−6.0 | 4800/450/−12.0 |
| noise | 0.15 | 0.10 | 0.10 | 0.00 | 0.00 |

`f0 = 560`'da F1'de sadece ~1.7 harmonik var → çığlık **kaynak harmonikleriyle** taşınır.
Bu yüzden scream'de `drive` (tanh) şart, ama §6'da anlatıldığı gibi *formant bankından sonra*
ve 2× oversample ile.

**4. PAIN GRUNT** — `f0 90–170`, `×0.88`, `hp 160`

| | F1 | F2 | F3 | F4 | F5 |
|---|---|---|---|---|---|
| F/BW/g | 550/130/0.0 | 1150/150/−1.0 | 2400/220/−5.0 | 3100/280/−9.0 | 3800/360/−14.0 |
| noise | 0.12 | 0.06 | 0.05 | 0.00 | 0.00 |

**5. BITE / CHOMP** — `f0 70–110` veya yok, `×0.80`, `hp 120`

| | F1 | F2 | F3 | F4 | F5 |
|---|---|---|---|---|---|
| başlangıç | 380/180/0.0 | 900/200/−3.0 | 1900/300/−8.0 | 2800/350/−12.0 | 3500/400/−18.0 |
| 60 ms sonra | 250/180/−2.0 | 640/200/−5.0 | 1500/300/−10.0 | 2200/350/−14.0 | 2900/400/−20.0 |
| noise | 0.20 | 0.15 | 0.10 | 0.00 | 0.00 |

Ağız kapanırken formantlar düşer — "chomp"un tüm formant hareketi bu 60 ms'lik glidedir.
Kemik kısmı bu bankın işi değil (modal bank + 3–6 kHz transient, §6).

**6. BREATH** — glottal yok, kaynak = turbulence, `×1.00`, `hp 100`

| | F1 | F2 | F3 | F4 | F5 |
|---|---|---|---|---|---|
| F/BW/g | 600/400/0.0 | 1400/500/−3.0 | 2600/700/−6.0 | 3400/800/−10.0 | 4200/900/−16.0 |
| noise | 1.00 | 1.00 | 0.80 | 0.40 | 0.20 |

`Q = F/BW ≈ 1.2–1.5`: bunlar rezonans değil, **geniş bantlar**. Nefesin "formantlı" olması
için yeterli, "vokal" olmaması için fazlasıyla geniş.

**7. DEATH RATTLE** — `f0 28–45`, `×0.80`, `hp 110`

| | F1 | F2 | F3 | F4 | F5 |
|---|---|---|---|---|---|
| F/BW/g | 350/220/0.0 | 700/260/−2.0 | 1500/350/−7.0 | 2300/450/−11.0 | 3000/550/−18.0 |
| noise | 0.25 | 0.20 | 0.15 | 0.10 | 0.05 |

BW'ler 2–3× insan: ölmekte olan doku = aşırı damping. Bu preset'te **voiced kaynak zayıf
olmalı** (gurgle ve rattle lider).

### 3.3 Zombi neden insan vokali değildir

| değişiklik | miktar | akustik sonuç | algı |
|---|---|---|---|
| larynx lowering | tüm F'ler **×0.80–0.90** | ΔF 1000 → 800–900 Hz | gövde 1.75 m → 2.1+ m |
| bandwidth widening | **×2–3** (F1 120–220) | rezonanslar sönümlü | "kalın", vokal ayırt edilemez |
| nasal coupling | pole 280–450 Hz (Q 4–8) + zero 500–900 Hz, −8…−12 dB | spektrumda çentik + alçak tepe | "burnu tıkalı", hayvani |
| sub-glottal (trakea) | pole 300–450 Hz, Q 3–6, −6…−10 dB | F1'in altında fazladan tepe | "içi boş", göğüs |
| "wet" duvar yükü | Q ×0.6, ekstra −2…−4 dB @ F3–F5 | yüksek formantlar söner | "ıslak", "etli" |
| false-fold ikinci kaynak | 0.5·f0, −12 dB | alt oktavda harmonik güç | "growl", "hırlama" |

Bu beşi birlikte uygulandığında "insan ama yanlış" elde edilir. Tek başına pitch düşürmek
(`f0 = 60 Hz`) sadece "kalın insan" verir; tek başına formant kaydırmak "büyük ama insan" verir.

### 3.4 Parallel mi cascade mi — bu proje için karar

| kriter | parallel | cascade (Klatt) |
|---|---|---|
| formant genlikleri bağımsız kontrol | **evet** | hayır (F/BW'den türer) |
| vokal doğallığı / F1–F2 etkileşimi | zayıf | **güçlü** |
| per-formant noise injection (wet katman) | **trivial** | zor (her BPF ayrı çıkar) |
| hızlı formant modülasyonunda kararlılık | **yüksek** | orta (kaskad kazanç birikir) |
| düşük f0'da "boomy" riski | var → HP ile çözülür | yok |
| kod satırı | ~25 | ~35 |

**Karar: parallel, varsayılan.** Gerekçe: (a) anlaşılabilirlik hedef değil, bu yüzden cascade'in
tek avantajı (doğal vokal) karşılıksız kalıyor; (b) wet/gurgle katmanı per-formant noise injection
istiyor; (c) formantlar 200–800 ms içinde hızla kayacak, kaskadda kazanç birikmesi riskli;
(d) tek tek formant genlikleri "snarl" karakterini taşıyor (F2 +2 dB, F1 0 dB gibi).

Cascade'i **sadece scream** için opsiyonel tut (`FormantMode::Cascade`): orada F1–F2 etkileşimi
"bağıran insan" hissini verir.

**Rezonatör (2-pole), paralel bank için:**

```
r   = exp(−π·BW/fs)
θ   = 2π·F/fs
b1  = 2·r·cos(θ)
c   = −r²
A   = (1 − r)·sqrt(1 − 2·r·cos(2θ) + r²)      // tepe kazancı = 1.0 (DOĞRU normalizasyon)
y[n] = A·g·x[n] + b1·y[n−1] + c·y[n−2]        // g = 10^(gain_db/20)
```

`A = 1 − b1 − c` (Klatt'ın klasik DC-normalizasyonu) **kullanılmamalı**: ölçtüm, F=500/BW=100'de
tepe kazancı 5.05, F=2500/BW=300'de 8.43 oluyor → F3 patlar. Yukarıdaki `A` ile tepe kazancı
tam 1.0000.

İki ölçülmüş uyarı:

- `r = exp(−π·BW/fs)` ilişkisi nominal BW'yi tam tutmaz: ölçülen −3 dB genişliği F=500/BW=100'de
  85 Hz, F=2500/BW=300'de 173 Hz, F=500/BW=400'de 573 Hz. Yani **0.6–0.9×**. Tablodaki BW'leri
  bu sapmayı bilerek gir; kritik yerlerde (F1) BW'yi %20 büyük seç.
- **F1'in görünmesi için kaynağı high-pass'ten geçir.** Ölçüm: `f0 = 72 Hz`, growl tablosu,
  HP yok → bant enerjisi 315 Hz'te 420 Hz'den **1.5 dB yüksek**; yani F1 lokal maksimum bile
  değil. Sebep: kaynak `h1–h3`'te hâlâ çok güçlü. `hp 150–250 Hz` (2 kutup) uygulandığında
  80 Hz bandı 315'e göre 10–15 dB düşer ve F1 tepe olur. Bu, gerçek trakeanın da yaptığı şeydir.

### 3.5 Formant geçişleri: statik vokal yerine "kelime"

Statik formant = "vokal pedalı" (org). Kelime hissi **geçiş hızından** gelir:

| geçiş süresi | algı | örnek |
|---|---|---|
| 5–25 ms | stop consonant (b, d, g) | "buh" onset'i |
| 25–60 ms | yarı-stop / affricate | "chomp" |
| **60–150 ms** | **glide (w, y) — hece sınırı** | growl'un "wr-" başlangıcı |
| 150–400 ms | diphthong | "aaa→ooo" |
| 400–800 ms | iki ayrı vokal | ölüm iniltisi |

Kurallar:

1. **En az iki formant birlikte hareket etsin.** Tek formant kaydırmak "filtre sweep" gibi duyulur
   (mevcut `Sweep` katmanının yaptığı şey). F1 ve F2 birlikte, zıt yönlerde hareket ederse
   "konuşma" olur.
2. **F1 sıçraması 50 ms'de 1 oktavı geçmesin**, yoksa "warble" olur.
3. **Segment + hold + glide** yapısı kullan: 120–250 ms sabit, 40–80 ms geçiş.
4. Bir **locus** ekle: stop'tan sonra F1 200–300 Hz'den başlar, 30–60 ms'de vokale çıkar.
   Tek başına en güçlü "konuşma" ipucu budur.

Growl için 3 segmentlik "kelime" (toplam 700 ms):

| t (ms) | F1 | F2 | F3 | niyet |
|---|---|---|---|---|
| 0 | 300 | 1650 | 2500 | kapalı ağız (onset) |
| 80 | 620 | 1050 | 2100 | açılma — "a" |
| 380 | 520 | 900 | 2000 | gövde |
| 620 | 380 | 1300 | 2400 | kapanma — "n" |

Scream (900 ms): F1 950→1150→800, F2 1750→1450→1900 (F1 ve F2 zıt yönde = "ağzı açıp bağırma").
Death rattle (1.8 s): F1 350→300→260, F2 700→620→560 (yavaş çöküş).

---

## 4. WET / GURGLING KATMANI

Boğazdaki sıvı, üç ayrı akustik olay üretir: (a) **bubble** rezonansları, (b) sıvı-katı sürtünmesi
(broadband burst), (c) kavitasyonun getirdiği hızlı genlik modülasyonu.

**Bubble frekansı (Minnaert):** `f = 3.26 / R` (R metre). Boğaz/yarık boyutları için:

| R | 0.5 mm | 1 mm | 2 mm | 3 mm | 5 mm | 8 mm |
|---|---|---|---|---|---|---|
| f | 6520 Hz | 3260 Hz | 1630 Hz | 1087 Hz | 652 Hz | 407 Hz |

Boğazda 2–8 mm kabarcıklar baskın → **400–1600 Hz**, tract tarafından bir kez daha süzülür.
Yani gurgle'ın "merkezi" F1'in hemen üstüdür, 300–900 Hz.

**Olay yapısı (tek bubble):**

```
f(t) = f0 · (1 + 0.35·(1 − e^(−t/8ms)))         // hızlı yukarı chirp (kabarcık büyüyor)
env(t) = (t/2ms)^1.5 · e^(−t/τ),  τ = 15…60 ms   // asimetrik: hızlı atak, uzun decay
```

**Katman reçetesi:**

| bileşen | parametre | not |
|---|---|---|
| bubble events | Poisson, `rate = 8–25/s` (canlı), 2–6/s (ölmek üzere) | IOI log-normal, σ = 0.5–0.7·mean |
| bubble frekansı | 200–1200 Hz, `f0 = F1 · U(0.8, 1.6)` | tract ile ilişkili olmalı |
| bubble süresi | 12–60 ms, `curve 2–3` | 12 ms = "tık", 60 ms = "glug" |
| cluster | her olaydan sonra %20 olasılıkla 15–40 ms içinde 2.–3. olay | **"sıvı" hissinin yarısı budur** |
| gap | %10 olasılıkla 250–600 ms sessizlik | makine değil, organizma |
| cavity ring | resonator 300–700 Hz, Q 8–14 (T60 60–150 ms) | her burst'ten sonra çınlar |
| liquid noise | pink, BP 400–1800 Hz, 8–30 ms, atak 2–4 ms | "squelch" |
| wet bed | pink → BP 500–900 Q 1.2, −24…−30 dB, AM 4–12 Hz %30–50 | sürekli ıslaklık |
| "wet" formant etkisi | F3–F5 gain −3 dB ek, Q ×0.7 | ağızda sıvı var |

**Neden "sıvı" duyulur — 5 ipucu (hepsi gerekli):**

1. Envelope asimetrisi: atak 2–4 ms, decay 30–60 ms (simetrik zarf = "tık").
2. Spectral centroid decay boyunca **düşer** (kabarcık büyür / boşluk dolar).
3. Olaylar **kümelenir** (unifrom Poisson bile "rain" gibi duyulur, cluster "gurgle" gibi).
4. Her olayın ardından 60–150 ms **kuyruk** (cavity ring) — kuru burst "click" olur.
5. Frekanslar tract formantlarıyla **ilişkili** (rastgele 5 kHz bubble = "su damlası", boğaz değil).

---

## 5. BREATH VE AIR

Nefes, "hiss" değildir; çünkü gerçek nefeste tract hâlâ rezonatördür ve akış yavaşça değişir.

**Spektral gerçekler:**

| nefes tipi | tepe bölgesi | eğim | üst sınır |
|---|---|---|---|
| sakin burun nefesi | 300–800 Hz | −6…−9 dB/oct | 4–6 kHz |
| ağız nefesi | 700–1500 Hz | −6 dB/oct | 6–8 kHz |
| zorlu/öfkeli nefes | 1.5–3 kHz | −3…−6 dB/oct | 10–12 kHz |
| wheeze (daralmış) | 0.9–2.5 kHz, Q 8–20 | dar bant | — |
| ıslık (whistle) | tek tepe, Q 20–40 | tonal | — |

**Yapı (hiss'ten ayıran şey):**

```
kaynak: white → 3 paralel BPF
   350–700 Hz   (göğüs)   gain 0 dB
   1200–2200 Hz (boğaz)   gain +2 dB
   3000–5500 Hz (dudak)   gain −6 dB
→ topla → LP 6–8 kHz (sert) / 10–12 kHz (zorlu)
→ formant bank (BREATH tablosu, §3.2-6, noise=1.0)
→ envelope: atak 40–120 ms, decay 150–600 ms
→ ripple: 3–8 Hz, %10–25 (akış düzensizliği)
→ wheeze: ayrı BPF 0.9–2.5 kHz Q 8–20, sadece daralma anında, −18…−26 dB
```

| parametre | değer | neden |
|---|---|---|
| atak | 40–120 ms | <20 ms = "click/pff", >200 ms = "rüzgâr" |
| decay | 150–600 ms | nefes 0.3–0.8 s sürer |
| HP | 100–150 Hz | göğüs gürültüsünü temizler |
| LP | 6–8 kHz | "hiss"in asıl kaynağı 8 kHz üstüdür |
| ripple | 3–8 Hz, %10–25 | statik zarf = "noise generator" |
| stereo | iki kanal farklı seed + 5–15 ms fark | tek mono noise "duvar" gibi duyulur |

**Whisper (horde için):** aynı zincir ama kaynak = glottal *yok*, sadece turbulence;
formant bank BREATH tablosu yerine HIGH SNARL tablosu (Q'lar 3–4'e çekilmiş) → "fısıldayan
kalabalık". f0 taşımadığı için pitch'i yok; bu yüzden 12+ örneği üst üste bindirmek gerekir.

---

## 6. AUDIT VE PLAN

### 6.1 Preset bazında: zayıf katman → yerine ne

Ortak hata yeniden: **voice katmanı her yerde en sessiz**, gürültü katmanı en yüksek. İkinci ortak
hata: hiçbir yerde formant bankı yok, "formant" diye Q=6 band-pass'lenmiş pembe gürültü var
(`render.rs`'de `f.set(pitch, 6.0)` sabit — yani BW = F/6, 420 Hz'de 70 Hz "çok dar",
2400 Hz'de 400 Hz "çok geniş"; tam tersi olmalı).

| preset | zayıf katman(lar) | sorun | yerine |
|---|---|---|---|
| **growl_low** | `glottal` Saw 92→61 @−7 | formant yok, saw, `fm 1.02` = detune | `Voice`: Rosenberg f0 74→62, `t1 .38 t2 .14 a_ret .30`, jitter 2.5 %, shimmer 10 %, HP 150 → LOW GROWL bankı |
| | `sub` sine 58→44 @−9 | drone, harmoniksiz | period-2 subharmonic `d=0.07` (kaynağın içinde) + false-fold kaynağı 37 Hz @−12, LP 400 |
| | `throat rasp` +**6 dB** | vokalden 13 dB yüksek, Q sabit | formant bankasına `noise 0.10` olarak gömülür, ayrı katman **−18 dB** |
| | `creak` +4 dB | velvet = 480 pps crackle, creak değil | 22–30 Hz pulse train, jitter %12, BP 300–900, −14 dB |
| | `breath` 0 dB | vokalle aynı seviyede | −16 dB, §5 zinciri |
| **growl_high** | `throat` Saw 155→118 @−7 | aynı | `Voice` f0 148→126, `t1 .42 t2 .12 a_ret .35`, jitter 3 %, HP 180 → HIGH SNARL |
| | `rasp` 0 dB | 7 dB vokalden yüksek | noise=0.10 olarak bankın içine, −16 dB |
| | `snarl` white BPF 2400 Q1.4 @−10 | 1.7 kHz geniş = hiss | nasal pole + F2 +2 dB; ayrıca roughness AM 55 Hz %30 |
| | `sub` 82 sine @−13 | gereksiz | period-2 `d=0.05`, f0/2 = 63 Hz |
| | `spit` −4 | tek başına gürültü | −20 dB, Poisson 6/s, 5 ms |
| **scream** | `voice` Saw 700→520 @−5 | F1 (950) 560 Hz'de sadece h1.7 → bank boş | `Voice` f0 620→540→480, `t1 .55 t2 .20 a_ret .15` (breathy LF), HP 200 → SCREAM bankı, cascade |
| | `formant` grain 1700 @+2 | gürültü, pitch yok | F2/F3 noise=0.12 olarak bankın içine |
| | `distorted edge` **square 350** @−10 | oktav-altı square = en güçlü robot ipucu | **sil**; yerine drive'ı formant bankından *sonra*, 2× oversample, 3.5 dB |
| | `tear`, `air`, `room` | hepsi gürültü, toplamı vokalden yüksek | `tear` −20, `air` → breath zinciri −18, `room` → gerçek reverb send (katman değil) |
| **pain_hit** | `grunt` Saw 180→105 @−6 | 90 ms'de %42 düşüş = "pew" | `Voice` f0 165→120 `pitch_tau 0.25` (daha yavaş), glottal stop onset (8 ms sessizlik + sert pulse), PAIN GRUNT bankı |
| | `throat rasp` **+3** | vokalden 9 dB yüksek | bankın içinde noise=0.12 |
| | `exhale` −2 | vokalle yarışıyor | −14 dB, §5 |
| | `wet click` +2 | en yüksek 2. katman | −18 dB, 2–3 bubble event |
| **attack_lunge** | `growl` **−10 dB** | en sessiz katman! | −2 dB, f0 120→165 (yükselen, "atılma"), formant geçişi 0→200 ms |
| | `rasp` −1 | vokali maskeliyor | −16 |
| | `swipe` −9 | duyulan tek şey bu | −12, ama `cloth` −18 ve pan ile |
| **bite** | `tooth snap` white BPF 1100 Q1.4 @**+6** | 1.1 kHz'de Q1.4 = "boom", click değil | transient 0.4 ms + HP 3 kHz + modal 4 kHz, +2 dB |
| | `bone` @**−42 dB** | duyulmuyor (ölü kod) | −14 dB, T60 45 ms, base 1.2 kHz (bone_modes) |
| | `squelch` | iyi ama statik | 600→1800 sweep korunur, üstüne BITE formant glide'ı (380→250) |
| | `tear` −12, `wet tail` −4 | gürültü ağırlıklı | −20 / −10 + 3 bubble cluster |
| **death_rattle** | `groan` Saw 78→38 @−7 | 38 Hz'de harmonikler yanlış yerde, saw "buzz" | `Voice` f0 72→40, **period-3 `d=0.12` artan**, jitter %8→%14, DEATH RATTLE bankı |
| | `fall`, `gurgle`, `rattle` | gürültü ağırlıklı, `gurgle` +2 | gurgle §4 reçetesi (cluster'lu), rattle 18/s |
| | `sub` 50→32 sine @−12 | gereksiz | kaldır; false-fold 24 Hz @−14 |
| **idle_breath** | `wheeze` **−31**, `chest rumble` **−31** | duyulmuyor (ölü kod) | −20 ve −22 |
| | `inhale` BPF 900 Q1.1 | 800 Hz geniş bant = "shhh" | §5 üç bantlı yapı + BREATH bankı |
| | tümü | formant yok, pitch yok | whisper varyantı: HIGH SNARL bankı, Q×0.4 |
| **horde_bed** | 3× Saw @−10/−11/−12 | üç synthesizer | 3× `Voice`, farklı `t1/t2/a_ret`, farklı jitter seed, tract ×0.80/0.86/0.92 (üç farklı boy) |
| | `rasp bed` **+1 dB** | hepsinden yüksek | −12, banka gömülü |
| | `sub drone` 44→36 @−12 | drone | kaldır; false-fold yatağı |

**Global düzeltmeler (preset'ten bağımsız):**

| sorun | düzeltme |
|---|---|
| `Saturator` 1× oversample, `drive_db 4–6` | 2× oversample (veya ADAA); yoksa −35 dB alias "dijital fizz" |
| master `presence` 2.5 dB @1.7 kHz | formant bankı geldiğinde **kaldır** (çift rezonsans) |
| `vibrato` 5.5 Hz sinüs | `pitch_drift`: 0.3–3 Hz random walk, ±3 %, **sinüs değil** |
| `fm_hz 1.02` | kaldır; yerine jitter (≥2 %) |
| Q=6 sabit grain filtresi | grain katmanları formant bankına gömülür; ayrı kullanılırsa `Q = F/BW` doğru hesaplanır |
| tüm noise katmanları vokalle eşit | noise/voice oranı: **−12…−20 dB** (growl), −8…−12 dB (scream) |

### 6.2 `VoiceSource` + `FormantBank` API

Yeni dosya: `crates/sg-dsp/src/voice.rs` (~150 satır), `lib.rs`'e `pub mod voice;`.

```rust
use crate::rng::Rng;

// ---------- 1. SOURCE ----------

#[derive(Clone, Copy, Debug)]
pub struct GlottalCfg {
    pub f0_hz: f32,          // 28..800
    pub t1: f32,             // open quotient, 0.30..0.55
    pub t2: f32,             // speed quotient, 0.08..0.25
    pub a_ret: f32,          // return lobe, 0.10..0.40
    pub ta: f32,             // return time, 0.005..0.060 (T0 cinsinden)
    pub jitter: f32,         // 0.00..0.15  (periyot başına bağıl std)
    pub shimmer: f32,        // 0.00..0.30
    pub sub_depth: f32,      // 0.00..0.35  period-doubling derinliği
    pub sub_div: u8,         // 2 veya 3
    pub aspiration: f32,     // 0.00..0.60  kaynağa karışan noise
    pub roughness_hz: f32,   // 0.0 (kapalı) veya 25..150
    pub roughness_depth: f32,// 0.00..0.80
}

impl Default for GlottalCfg { /* f0 80, t1 .40, t2 .16, a_ret .30, ta .02, jitter .025,
                                 shimmer .10, sub .06/2, aspiration .05, rough 0/0 */ }

pub struct GlottalVoice {
    table: Vec<f32>,   // bir periyot, 2048, DC'siz, normalize
    sr: f32,
    phase: f32,
    cfg: GlottalCfg,
    period_mul: f32,   // periyot başına jitter
    period_amp: f32,   // periyot başına shimmer
    sub_i: u8,         // subharmonic sayaç
    sub_phase: f32,    // roughness LFO fazı
}

impl GlottalVoice {
    /// `f0_max` = preset'in en yüksek f0'ı (tablo aliasing'inı önlemek için).
    pub fn new(sr: f32, cfg: GlottalCfg, _f0_max: f32) -> Self;
    pub fn set_f0(&mut self, hz: f32);                 // 28..800, log-domain'de yumuşat
    pub fn set_cfg(&mut self, cfg: GlottalCfg);        // tablo yeniden kurulur
    pub fn next(&mut self, rng: &mut Rng) -> f32;      // mono, [-1,1]
    pub fn reset(&mut self);
}

// ---------- 2. TRACT ----------

#[derive(Clone, Copy, Debug)]
pub struct Formant { pub f: f32, pub bw: f32, pub gain_db: f32, pub noise: f32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormantMode { Parallel, Cascade }

#[derive(Clone, Copy, Debug)]
pub struct FormantTable {
    pub name: &'static str,
    pub f: [Formant; 5],
    pub tract: f32,        // 0.70..1.20 ölçek (0.85 = büyük yaratık)
    pub hp_hz: f32,        // 100..250, kaynak high-pass (F1'i görünür yapar)
    pub tilt_db_oct: f32,  // -6..0 ek tilt
    pub nasal: Option<(f32, f32)>,  // (pole_hz, gain_db)
    pub subglottal: Option<(f32, f32)>,
}

pub struct FormantBank {
    // 5 paralel/kaskad rezonatör + glide state
}

impl FormantBank {
    pub fn new(sr: f32) -> Self;
    pub fn load(&mut self, table: &FormantTable);          // katsayıları kurar
    pub fn set_mode(&mut self, m: FormantMode);
    /// 5 formantın hedef (f, bw) çiftine doğru `secs` içinde üstel glide.
    pub fn glide_to(&mut self, targets: &[(f32, f32); 5], secs: f32);
    pub fn stretch(&mut self, k: f32);                     // tract ölçeği, 0.70..1.20
    pub fn process(&mut self, x: f32, noise: f32) -> f32;  // noise: 0..1 (aspiration)
    pub fn process_block(&mut self, io: &mut [f32], noise: &[f32]);
    pub fn reset(&mut self);
}

// ---------- 3. WET ----------

#[derive(Clone, Copy, Debug)]
pub struct GurgleCfg {
    pub rate_hz: f32,      // 2..25
    pub f_lo: f32,         // 200
    pub f_hi: f32,         // 1200
    pub decay_s: f32,      // 0.015..0.060
    pub cluster_p: f32,    // 0.20
    pub ring_hz: f32,      // 300..700
    pub ring_q: f32,       // 8..14
    pub noise_db: f32,     // -30..-18
    pub follow_f1: bool,   // bubble frekansını F1'e bağla
}

pub struct Gurgle { /* Poisson scheduler + ring resonator */ }
impl Gurgle {
    pub fn new(sr: f32, cfg: GurgleCfg) -> Self;
    pub fn next(&mut self, rng: &mut Rng) -> f32;
    pub fn set_follow(&mut self, f1_hz: f32);   // tracts formantına izle
}
```

**Engine tarafı (tek satırlık preset):**

```rust
// crates/sg-engine/src/layer.rs
pub enum LayerKind {
    // ... mevcutlar ...
    /// Full source-filter voice: glottal + formant tract + wet layer.
    Voice(Box<VoiceParams>),
}

pub struct VoiceParams {
    pub glottal: GlottalCfg,
    pub formants: FormantTable,
    pub segments: Vec<FormantSegment>,  // boş = statik
    pub gurgle: Option<GurgleCfg>,
    pub env: AdEnv,
    pub noise_level_db: f32,            // aspiration, -30..-6
}

pub struct FormantSegment { pub at_s: f32, pub targets: [(f32, f32); 5], pub glide_s: f32 }
```

Preset yazımı (mevcut DSL ile aynı his):

```rust
r.layer(
    Layer::new("voice", LayerKind::Voice(Box::new(VoiceParams {
        glottal: GlottalCfg { f0_hz: 74.0, t1: 0.38, t2: 0.14, a_ret: 0.30, jitter: 0.025,
                              shimmer: 0.10, sub_depth: 0.07, sub_div: 2, ..Default::default() },
        formants: tables::LOW_GROWL,
        segments: vec![
            FormantSegment { at_s: 0.00, targets: [(300.,110.),(1650.,180.),(2500.,230.),(3200.,280.),(4000.,350.)], glide_s: 0.06 },
            FormantSegment { at_s: 0.08, targets: [(620.,150.),(1050.,170.),(2100.,240.),(2900.,300.),(3600.,380.)], glide_s: 0.07 },
            FormantSegment { at_s: 0.62, targets: [(380.,140.),(1300.,170.),(2400.,240.),(3000.,300.),(3700.,380.)], glide_s: 0.09 },
        ],
        gurgle: Some(GurgleCfg { rate_hz: 12.0, follow_f1: true, ..Default::default() }),
        env: AdEnv::swelled(0.070, 1.15),
        noise_level_db: -18.0,
    }))),
).gain(-2.0)
```

### 6.3 Uygulama sırası (150 satır bütçesi)

| # | iş | satır | test |
|---|---|---|---|
| 1 | `GlottalTable` (Rosenberg + return + DC + normalize) | 30 | f0 80'de ölçülen eğim −8.8 dB/oct (±1) |
| 2 | jitter/shimmer/subharmonic/roughness wrapper | 30 | jitter 2.5 % → periodicity 0.94 |
| 3 | `FormantBank` parallel + `A` normalizasyonu | 30 | tepe kazancı 1.000 ± 0.01 |
| 4 | glide + tract stretch + nasal/subglottal pole | 25 | F1 420→620 geçişi 70 ms'de |
| 5 | `Gurgle` (Poisson + cluster + ring) | 20 | 12/s'de 1.5 s'de 14–22 olay |
| 6 | `LayerKind::Voice` + `tables::*` sabitleri | 15 | §0'daki 8 bant tablosunda F1/F2/F3 lokal maksimum |

Kabul kriteri (ölçülebilir): her preset için 8-bant tablosunda F1, F2, F3 **lokal maksimum**
olmalı ve periodicity **> 0.80**. Mevcut durum: hiçbir preset bunu sağlamıyor (§0).

---

## 7. Kaynaklar

- Fant, G., Liljencrants, J., Lin, Q. (1985), *A four-parameter model of glottal flow*, STL-QPSR 26(4).
  LF formülleri, `α = −ω_g·cot(ω_g·Te)` koşulu, `Ta` return phase.
- Rosenberg, A. (1971), *Effect of glottal pulse shape on the quality of natural vowels*, JASA 49(2).
  §1.2'deki iki-branch pulse ve `T1/T2` oranları.
- Klatt, D. (1980), *Software for a cascade/parallel formant synthesizer*, JASA 67(3).
  `cascade` vs `parallel`, formant bandwidth tabloları, paralel branch kazanç normalizasyonu.
- Terhardt, E. (1974), *On the perception of periodic sound fluctuations (roughness)*, Acustica 30.
  Roughness'ın 50–90 Hz tepe bölgesi.
- Fant, G. (1995), *A four-parameter model of glottal flow* (Rd parametresi) — `t1/t2/a_ret`
  haritası için.
- Titze, I. (1994), *Principles of Voice Production* — jitter/shimmer normal değerleri
  (%0.5–1.0 / %3–4), false-fold mekanizması.
- Minnaert, M. (1933), kabarcık rezonansı `f = 3.26/R` — gurgle frekans tablosu.

**Bu repoda ölçülenler** (§0, §1.1, §2.2, §3.4'teki sayılar): `sg-presets` presetleri
`sg_engine::render` + Svf band-pass oktav analizi; Rosenberg/LF zarfı 8192-nokta DFT;
periodicity normalize edilmiş autocorrelation. Ölçüm kodu geçici bir integration test
ile çalıştırıldı ve silindi — dokümandaki sayılar tekrar üretilebilir.
