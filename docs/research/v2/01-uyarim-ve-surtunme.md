# 01 — Uyarım ve Sürtünme: Temas Fiziğinden Ses Üretimi

> **Kapsam:** Bir darbe ya da sürtünme olayının *kaynak* tarafı — yani modal banka giren sinyal.
> Rezonatör tarafı (modal sentez, filtreler, reverb) 01-dsp-sentez.md'de; burada yalnızca **uyarım** var.
> **Hedef okuyucu:** yarın `sg-dsp/src/excite.rs` yazacak mühendis. Her blok ≤120 satır Rust, sıfır yeni bağımlılık.
> **Sayısal varsayım:** `fs = 48 000`, aksi belirtilmedikçe. `Tc` = temas süresi (s), `J` = momentum transferi (N·s).
>
> **Kardeş dokümanlar:** `v2/02-modal-malzeme.md` rezonatör tarafını (malzeme mod tabloları, pickup, IR-enerji
> normalizasyonu, `ModalBank::excite(force)` arayüzü) işliyor. Bu doküman o arayüze **ne gireceğini** tanımlar.
> İki dokümanın kesiştiği tek yer `g_k` normalizasyonu (§1.4 burada, uygulama 02 §3.2'de) — çelişki yok.
>
> **Teşhis (neden şu an "sentetik"):** mevcut motorun her darbesi *aynı jest*: gürültü patlaması → lineer rampa →
> modal bank → sine pitch-drop. Gerçek bir darbede sesin %80'i **temas süresinin ürettiği spektrumdan** gelir ve
> temas süresi malzemeye, kütleye, hıza bağlıdır. Motorun `velocity` diye bir parametresi yok; her vuruş aynı enerjiyle
> oluyor. Bu doküman o boşluğu doldurur.

---

## 0. Yönetici Özeti — 7 karar

| # | Karar | Neden |
|---|---|---|
| K1 | Uyarım **impuls değil kuvvet** olacak; kuvvetin *alanı* = `J = m·v·(1+e)` | Ses yüksekliği momentumla, parlaklık temas süresiyle ölçeklenir. İkisi bağımsız kontrol edilebilir hale gelir. |
| K2 | `Tc` Hertz temasından hesaplanacak, `Tc = 2.94·(5m/4k)^0.4·v^-0.2` | Tek formül, tüm darbe sınıflarını (bilya 26 µs → yumruk 15 ms) kapsıyor. |
| K3 | Her darbede **iki ölçekli temas**: makro `Tc` (gövde) + asperite mikroteması (click) | Yumuşak cisimlerin makro `Tc`'si 10 ms ama duyulan "tık" 200 µs. Tek ölçekle ikisi de olmuyor. |
| K4 | Modal kazanç **`g_k = sin θ_k`** (impuls) veya `sqrt(1-r_k²)` (gürültü) | Mevcut kodda `g=1`; bu, her moda örtük `1/sinθ` kazancı yani **-6 dB/oct yanlış tilt** veriyor. |
| K5 | `StickSlip` + `GranularFriction` **yeni blok** olarak girecek | Motorda sürtünme modeli *yok*; creak katmanı sawtooth + 5.5 Hz vibrato. |
| K6 | Lineer rampa attack yasak; **raised-cosine 0.05–0.2 ms** | `env.rs` `perc()` 0.5 ms lineer rampa: 1.2 ms'lik crack'in %40'ını yiyor ve türev süreksizliği ekliyor. |
| K7 | Nonlineerlik **katman başına**, master'da değil; DC üreten asimetri yasak | `fx.rs` `Saturator` her sese +0.064 DC basıyor → 4 ms LF thump + headroom kaybı. |

---

## 1. DARBE / TEMAS UYARIMI

### 1.1 Hertz temas modeli ve temas süresi

İki elastik cisim, efektif eğrilik yarıçapı `R`, efektif modül `E*`:

```
1/E* = (1-ν1²)/E1 + (1-ν2²)/E2
k    = (4/3) · E* · sqrt(R)          [N/m^1.5]     (küre–düzlem)
F(δ) = k · δ^1.5
```

Hareket denklemi `m·δ̈ = -k·δ^1.5`. Enerji korunumu (`½mv² = ∫F dδ = (2/5)kδmax^2.5`) verir:

```
δmax = (5·m·v² / (4·k))^(2/5)
Tc   = 2.94 · (5·m / (4·k))^(2/5) · v^(-1/5)        [s]
```

**Kritik gözlem:** `Tc ∝ v^(-0.2)`. Hız 100 kat artarsa temas süresi yalnızca **2.5 kat** kısalır.
Yani "sert vurdum → parlak" etkisinin *büyük kısmı* `Tc`'den gelmez. Gerçek mekanizma §1.5'te.

Sayısal tablo (formülle hesaplanmış, elle yazılmadı):

| Temas | m | R | E* | v | **Tc** | δmax | J |
|---|---|---|---|---|---|---|---|
| Çelik bilya 5 mm → çelik | 2 g | 5 mm | 115 GPa | 1.0 m/s | **25.9 µs** | 8.8 µm | 4.0 mN·s |
| Cam şişe → fayans | 0.4 kg | 30 mm | 38 GPa | 2.0 m/s | **204 µs** | 139 µm | 1.6 N·s |
| Plastik kalem kapağı → masa | 3 g | 4 mm | 1.65 GPa | 0.4 m/s | **209 µs** | 28 µm | 2.4 mN·s |
| Çelik çekiç → meşe | 0.5 kg | 25 mm | 12.5 GPa | 3.0 m/s | **334 µs** | 341 µm | 3.0 N·s |
| Yumruk → et | 2 kg | 50 mm | 2 MPa | 6.0 m/s | **14.5 ms** | 29.6 mm | 24 N·s |
| Bot topuğu → beton (kauçuk taban) | 8 kg | 40 mm | 16.7 MPa | 0.5 m/s | **18.6 ms** | 3.2 mm | 8.0 N·s |

> **Uyarı:** son iki satırdaki 15–19 ms, *makro* kuvvet darbesidir. Duyulan "tık" bundan gelmez (bkz. §1.2).
> Ayrıca Hertz modeli pürüzsüz bir küre varsayar; gerçek yüzeyde 10–500 asperite aynı anda temas eder.

### 1.2 İki ölçekli temas — bu raporun en önemli fikri

Gerçek bir yüzey pürüzlüdür. Temas, önce 1–100 µm ölçeğinde birkaç asperitede başlar, sonra yük arttıkça
temas alanı büyür. Sonuç: **kuvvet sinyali iki bileşenin toplamıdır.**

| Bileşen | Ölçek | Tc | Ne üretir | Nereye gider |
|---|---|---|---|---|
| **Asperite** (mikrotemas) | 5–100 µm | **30–300 µs** | broadband "tık/crack", 2–15 kHz | yüksek modlar + HF click katmanı |
| **Makro** (gövde teması) | mm–cm | 0.2–20 ms | gövde, "thump", 100–1500 Hz | düşük modlar |

Duyulan parlaklık neredeyse tamamen **asperite** tarafından belirlenir. Bu yüzden:

- Sert yüzeylerde (çelik, cam, beton) asperite teması *kırılgan* → mikro-temaslar kopar, her biri ayrı bir
  impuls üretir → ses "çıtır çıtır + parlak" olur.
- Yumuşak yüzeylerde (et, kumaş, çamur) asperiteler *yield* olur, tek bir yumuşak temasa birleşir → ses
  "boğuk, ıslak, tek vuruş" olur.

**Kod sonucu:** `ImpactExcitation` iki puls üretir: `force_macro` (Hertz, `Tc`) ve `force_asp` (1–4 örnek,
`asperity` parametresiyle ölçekli). Çağıran taraf makro'yu modal banka, asperite'yi HF click filtresine verir.

### 1.3 Kuvvet pulsunun spektrumu — `Tc` gerçekten nereyi kesiyor?

Hertz kuvvet geçmişini `F(u) = (1-(2u-1)²)^1.5`, `u = t/Tc` ile modelleyip spektrumunu sayısal olarak
hesapladım (**tüm puls şekilleri için normalize**; `1/Tc` biriminde):

| Puls şekli | -3 dB | -6 dB | -10 dB | -20 dB |
|---|---|---|---|---|
| **Hertz `(1-(2u-1)²)^1.5`** | **0.634/Tc** | 0.876/Tc | **1.093/Tc** | 1.407/Tc |
| Yarım sinüs | 0.594/Tc | 0.819/Tc | 1.020/Tc | 1.304/Tc |
| Üçgen | 0.637/Tc | 0.885/Tc | 1.114/Tc | 1.476/Tc |
| Dikdörtgen | 0.442/Tc | 0.602/Tc | 0.738/Tc | 0.908/Tc |

**Kullanılacak ampirik yasalar** (Hertz şekli için):

```
f_-3dB  = 0.634 / Tc        f_-10dB = 1.09 / Tc        f_-20dB = 1.41 / Tc
```

Bu, tablodaki `Tc`'lerle:

| Temas | Tc | f_-3dB | f_-20dB | Yorum |
|---|---|---|---|---|
| Çelik bilya → çelik | 25.9 µs | 24.5 kHz | 54 kHz | Nyquist'in üstünde → **spektrum düz**, malzeme belirler |
| Cam şişe | 204 µs | 3.1 kHz | 6.9 kHz | cam modları 2–8 kHz'de → hepsi uyarılır |
| Kalem kapağı | 209 µs | 3.0 kHz | 6.7 kHz | |
| Çekiç → meşe | 334 µs | 1.9 kHz | 4.2 kHz | ahşap modları 200–1.5 kHz → hepsi uyarılır, >4 kHz kesik |
| Yumruk → et | 14.5 ms | 44 Hz | 97 Hz | **tek başına felaket**: 15 ms'lik puls Hiçbir şey uyarmaz |

> Tablo `v = 1 m/s` referansında. §1.7'deki `prepare` en yüksek velocity'de `v = 5 m/s`'ye çıkar, yani
> `Tc` tablodakinin `5^-0.2 = 0.725` katı olur (çekiç: 334 → 220 µs). §1.5'teki ölçüm tablosunda bu görünüyor.

Son satır K3'ün kanıtı: yumruğu yalnız makro Hertz ile modellemek, 44 Hz'lik bir alçak geçiren verir → duyulan
şey yok. Gerçek yumruk sesi, makro pulsun üstündeki **asperite** temasından gelir. Bu yüzden iki ölçek zorunlu.

### 1.4 Impuls mü, kuvvet mi? Ayrık zaman formülasyonu

Modal rezonatör (paralel ikinci derece), sürekli: `ÿ + 2σẏ + (σ²+ω²)y = γ·f(t)`.

Ayrık zaman (impulse-invariant, kutup-yarıçapı formu):

```
θ_k = 2π f_k / fs
r_k = exp(-6.907755 / (T60_k · fs))          // -60 dB kutup yarıçapı
a1  = 2 r cos θ ,  a2 = -r²

y_k[n] = a1·y_k[n-1] + a2·y_k[n-2] + g_k·x[n]
```

**Impuls cevabı** (birim örnek için): `h_k[n] = g_k · r^n · sin((n+1)θ) / sin θ`.

Buradan **iki farklı normalizasyon** çıkar ve ikisi farklı soruya cevap verir:

| Uyarım tipi | Doğru `g_k` | Sonuç |
|---|---|---|
| **Impuls / kuvvet pulsunun alanı sabit** | `g_k = sin θ_k ≈ 2π f_k/fs` | her mod **eşit tepe genlik** → spektrum `amp_k` tarafından belirlenir |
| **Sürekli gürültü** | `g_k = sqrt(1 - r_k²)` | her mod **eşit güç** → spektrum `amp_k` tarafından belirlenir |

> **Mevcut koddaki hata:** `modal.rs:59` `g = 1` kullanıyor, yani `g_k = 1` → implicit `1/sinθ_k` kazancı.
> `1/sinθ ∝ 1/f` → her modal bank **-6 dB/oct yanlış tilt** ile doğuyor. 180 Hz modu, 2.1 kHz modundan
> `sin(0.275)/sin(0.0236) = 11.5×` (21 dB) daha yüksek çıkıyor. Presetlerdeki `exciter_hp: 300–600`,
> `presence_db: +2.5 @ 3.2 kHz`, `drive_db: 3` değerlerinin hepsi bu tilti telafi etmek için var. Tilt
> düzeltilince o EQ'lar *fazla* parlak yapacak — birlikte değiştirilmeli.

**Kuvvet → impuls eşdeğerliği.** Kuvvet pulsunun *örnek-toplamı* normalize edilirse (Σx[n] = 1), çıkış
birim impuls cevabıyla **birebir aynı** olur; `Tc` yalnızca pulsun şeklini/spektrumunu değiştirir, seviyeyi
değiştirmez. Bu, K1'in pratik anlamı:

```rust
// doğru yaklaşım: toplamı 1'e normalize et, sonra J ile ölçekle
let s: f32 = w[..len].iter().sum();          // w = Hertz şekli
x[i] = J * w[i] / s;                          // J = m·v·(1+e)   [N·s]
// bankaya giren sinyal artık "kuvvet" değil, "momentum-normalize uyarım"
```

Böylece `velocity` **yalnızca J'yi** (yükseklik), `stiffness`/`mass` **yalnızca Tc'yi** (parlaklık) sürer.
Şu anki kodda ikisi de `gain_db`'ye kaçıyor.

Sub-sample `Tc` (çelik bilya: 1.24 örnek) tamamen güvenlidir — ayrık bir impulsun Nyquist üstü enerjisi
yoktur, katlanma olmaz. `len = max(2, round(Tc·fs))` yeterli. **Oversampling yalnızca nonlineer
aşamalarda gerekir** (§4).

### 1.5 Velocity → parlaklık eşlemesi (somut)

Fizik `Tc ∝ v^-0.2` diyor: 0.1 → 5 m/s arası (50× hız) yalnızca 2.2× parlaklık. Gerçekte 20 dB'lik bir
genlik artışı kulakta 4–6× parlaklık gibi okunur. Farkın kaynağı üç mekanizma:

1. **Eşik etkisi:** yüksek modlar zaten uyarılıyor ama düşük seviyede gürültü tabanının altında kalıyor.
   20 dB artış, 2–6 kHz bandındaki modları duyulur hale getirir. (Baskın mekanizma.)
2. **Temas sertleşmesi:** gerçek malzemelerde `E` gerinim hızıyla artar → `Tc` fizikten daha hızlı kısalır.
3. **Asperite kopması:** yüksek hızda daha çok asperite kırılır → daha çok mikro-impuls.

Pratik eşleme (0..1 `velocity`, ses tasarımı için kalibre edilmiş):

```
v      = 0.08 + 4.92 · velocity^2.5                   // 0..1 → 0.08 .. 5.0 m/s  (≈36 dB enerji aralığı)
Tc     = Tc_ref · (v_ref/v)^0.2 · (1 - 0.45·hardness)  // sertlik teması kısaltır
J      = m · v · (1 + restitution)                     // asıl yükseklik kontrolü
n_asp  = round(2 + 10 · velocity · asperity)           // asperite sayısı 2..12
f_tilt = 1200 · (v/v_ref)^0.35 · (0.4 + hardness)      // modal bank öncesi ek tilt (Hz)
```

Aşağıdaki tablo **§1.7'deki kodun gerçek çıktısıdır** (çekiç → meşe: `m=0.5 kg, R=25 mm, E*=12.5 GPa,
e=0.5, hardness=0.6, asperity=0.7, jitter=0.3`, 48 kHz; kod derlendi ve çalıştırıldı):

| velocity | v (m/s) | J | J (dB) | Tc | f_-3dB | örnek | Σforce/J | asperite RMS |
|---|---|---|---|---|---|---|---|---|
| 0.00 | 0.08 | 0.059 | −36.0 | 562 µs | 1129 Hz | 27 | **1.000** | −52.8 dB |
| 0.30 | 0.32 | 0.241 | −23.8 | 346 µs | 1831 Hz | 17 | **1.000** | −42.9 dB |
| 0.60 | 1.45 | 1.060 | −10.9 | 290 µs | 2185 Hz | 14 | **1.000** | −28.6 dB |
| 0.85 | 3.36 | 2.590 | −3.2 | 248 µs | 2553 Hz | 12 | **1.000** | −21.8 dB |
| 1.00 | 5.00 | 3.736 | 0.0 | 220 µs | 2884 Hz | 12 | **1.000** | −17.9 dB |

**Bu tablodan çıkan kritik sonuç — K3'ün kanıtı:**

- `Tc` yalnızca **1.35 oktav** değişiyor (562 → 220 µs). Yani `Tc` tek başına "sert vurdum → parlak" etkisinin
  ancak ~%25'ini açıklıyor.
- `asperite` enerjisi **35 dB** değişiyor (−52.8 → −17.9 dB). Parlaklığın gerçek taşıyıcısı bu.
- `Σforce/J = 1.000` her satırda: momentum korunuyor, yani velocity katmanları arasında seviye
  monoton ve öngörülebilir. (Asperite ayrı akışa alınmasaydı bu değer 0.875–1.04 arasında oynuyordu →
  velocity 0.85 sesi 1.0'dan 1.5 dB **daha yüksek** çıkıyordu; duyulabilir bir hata.)

> **Not:** `velocity^2.2` eğrisi bilinçli. Lineer `J ∝ v` müzikal olarak "komprese" hissettirir; kulak
> enerjiyi değil `v^1.5..v^2` gibi algılar. Oyun motorundan gelen lineer 0..1 değeri bu eğriden geçmeli.

### 1.6 Çoklu temas / takırtı (rattling)

Bir nesne düştüğünde tek darbe yoktur: 3–40 temas, azalan genlikle. İki farklı istatistik var:

**a) Bounce dizisi (deterministik, düzenli):** esnek çarpışma, `e` = restitution.
```
t_{k+1} = t_k + 2·v_k·(1+e)/(g·(1-e)) ...  basitleştirilmiş:
Δt_k = Δt_0 · e^k            (e = 0.55..0.85)
A_k  = A_0 · e^(2k)          (enerji ∝ v², her çarpışmada e² korunur)
```
`e = 0.7` → 5 temas sonra genlik -18 dB, süre 4.2×. Kovan, zıplayan taş, düşen vida.

**b) Takırtı (Poisson, düzensiz):** gevşek parçalar birbirine çarpıyor.
```
Δt = -ln(u) / λ(t) ,   λ(t) = λ0 · exp(-t/τ)      λ0 = 40..400 temas/s, τ = 30..150 ms
A  = A_0 · exp(-t/τ_a) · lognormal(0, 0.35)
```
Poisson şart: **sabit aralık kullanmak "machine gun" hissi verir** — 01-dsp-sentez §9.4'teki grain
kodunda bu doğru yapılmış, aynı mantık temas zamanlamasına da uygulanmalı.

**Neden Poisson:** insan kulağı 8–20 ms aralıklı düzenli darbeleri *ton* olarak duyar (flutter/tarak efekti).
Düzensizlik, tek tek temasları ayırt edilebilir kılar ve "gevşek parça" algısını üretir.

### 1.7 `ImpactExcitation` — blok tasarımı

```rust
// sg-dsp/src/excite.rs
use crate::rng::Rng;

#[derive(Clone, Copy, Debug)]
pub struct ImpactParams {
    pub mass_kg: f32,        // 0.001 .. 20
    pub radius_m: f32,       // 0.001 .. 0.10
    pub e_star_pa: f32,      // 1e6 (et) .. 2e11 (çelik)
    pub restitution: f32,    // 0.0 (et/kurşun) .. 0.95 (çelik)
    pub hardness: f32,       // 0..1  mikro-temas kırılganlığı
    pub asperity: f32,       // 0..1  mikro-temas enerji oranı
    pub contact_jitter: f32, // 0..1  Tc ve J varyansı
}

pub struct ImpactExcitation {
    sr: f32,
    pub p: ImpactParams,
    tc: f32, j: f32,          // vuruş başına hesaplanan
    n: u32, len: u32, inv_sum: f32,
    asp_next: f32, asp_left: u32, asp_amp: f32,
    done: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Prepared {
    pub velocity: f32,
    pub v_mps: f32,
    pub impulse: f32,      // N·s → modal bank giriş ölçeği
    pub contact_s: f32,
    pub f_minus_3db: f32,
    pub f_tilt_hz: f32,
}

#[inline]
fn hertz_shape(u: f32) -> f32 {
    let a = 1.0 - (2.0 * u - 1.0) * (2.0 * u - 1.0);
    if a <= 0.0 { 0.0 } else { a * a.sqrt() }
}

impl ImpactExcitation {
    pub fn new(sr: f32, p: ImpactParams) -> Self {
        Self { sr, p, tc: 0.0, j: 0.0, n: 0, len: 0, inv_sum: 1.0,
               asp_next: 0.0, asp_left: 0, asp_amp: 0.0, done: false }
    }

    /// Hertz temas süresi — §1.1.
    #[inline]
    pub fn contact_time(mass: f32, radius: f32, e_star: f32, v: f32) -> f32 {
        let k = (4.0 / 3.0) * e_star * radius.sqrt();
        2.94 * (5.0 * mass / (4.0 * k)).powf(0.4) * v.max(0.01).powf(-0.2)
    }

    /// 0..1 velocity → (v, J, Tc, f_-3dB, f_tilt). §1.5 tablosunu üreten kod.
    pub fn prepare(&mut self, velocity: f32, rng: &mut Rng) -> Prepared {
        let vel = velocity.clamp(0.0, 1.0);
        let v = 0.08 + 4.92 * vel.powf(2.5);                  // m/s
        let v_ref = 1.0;
        let tc0 = Self::contact_time(self.p.mass_kg, self.p.radius_m, self.p.e_star_pa, v_ref);
        let harden = 1.0 - 0.45 * self.p.hardness;
        let tc = tc0 * (v_ref / v.max(0.01)).powf(0.2) * harden;
        let jit = self.p.contact_jitter;
        let tc = tc * (1.0 + rng.normal() * 0.18 * jit);
        let j = self.p.mass_kg * v * (1.0 + self.p.restitution) * (1.0 + rng.normal() * 0.06 * jit);

        self.tc = tc.clamp(2.0 / self.sr, 0.060);            // 2 örnek .. 60 ms
        self.j = j;
        self.len = ((self.tc * self.sr).round() as u32).max(2);
        self.n = 0;

        // midpoint örnekleme: endpoint kullanılırsa len==2 puls tamamen sıfır çıkar
        let mut s = 0.0f32;
        for i in 0..self.len {
            s += hertz_shape((i as f32 + 0.5) / self.len as f32);
        }
        self.inv_sum = 1.0 / s;

        self.asp_left = (2.0 + 10.0 * vel * self.p.asperity).round() as u32;
        // asperite, makro pulsun TEPESİNE göre ölçeklenir (inv_sum), J'ye göre değil
        self.asp_amp = 0.35 * self.p.asperity * (0.5 + 0.5 * self.p.hardness) * self.inv_sum;
        self.asp_next = 0.0;
        self.done = false;

        Prepared {
            velocity: vel, v_mps: v, impulse: j,
            contact_s: self.tc,
            f_minus_3db: 0.634 / self.tc,
            f_tilt_hz: 1200.0 * (v / v_ref).powf(0.35) * (0.4 + self.p.hardness),
        }
    }

    /// (force, asperity). `force` toplamı **tam olarak** J'dir (momentum korunumu);
    /// `asperity` ayrı akış, HF click katmanına gider. İkisini toplamak momentumu
    /// bozar (bipolar mikro-darbeler birbirini götürür → katmanlar arası 1-2 dB oynama).
    #[inline]
    pub fn next(&mut self, rng: &mut Rng) -> (f32, f32) {
        if self.done { return (0.0, 0.0); }
        let mut force = 0.0f32;
        let mut asp = 0.0f32;
        if self.n < self.len {
            let u = (self.n as f32 + 0.5) / self.len as f32;
            force = hertz_shape(u) * self.inv_sum;
        }
        if self.asp_left > 0 && self.n as f32 >= self.asp_next {
            asp = rng.bipolar() * self.asp_amp * (1.0 + rng.normal() * 0.4);
            self.asp_left -= 1;
            let span = self.len as f32 * 0.4 + 1.0;
            self.asp_next = self.n as f32 + rng.next_f32() * span * 0.25;
        }
        self.n += 1;
        if self.n >= self.len && self.asp_left == 0 { self.done = true; }
        (force * self.j, asp * self.j)
    }
}
```

**Çıktı sözleşmesi:**

| Çıkış | Tip | Anlam | Kullanım |
|---|---|---|---|
| `next().0` | f32 | normalize kuvvet; **Σ = J tam** | modal bank girişi (impuls normalizasyonu ile) |
| `next().1` | f32 | asperite akışı, ayrı, DC'siz | HF click: `HP 3 kHz` + 4–12 ms decay |
| `impulse` | N·s | `m·v·(1+e)` | bank kazancı: `g_k = sinθ_k · J` |
| `contact_s` | s | Hertz `Tc` | QA / log |
| `f_minus_3db` | Hz | `0.634/Tc` | otomatik `exciter_lp` — **artık elle girilmez** |
| `f_tilt_hz` | Hz | hız/sertlik tilt eğrisi | `amp_k *= (f_k/f_tilt)^-β` |

> **Neden iki akış:** ilk sürümde asperite `force`'a toplanıyordu; ölçüm `Σ/J` değerini 0.875–1.04 arasında
> oynattı (bipolar darbeler kısmen birbirini götürüyor). Bu, velocity katmanları arasında **monoton olmayan**
> 1.5 dB'lik seviye hatası demek — yani "daha sert vurdum, daha kısık çıktı". Ayırmak hem momentumu
> birebir korur hem de asperite'yi istediği kadar "tıklı" yapma serbestliği verir.

`asperity` sinyali modal banktan *önce* ayrılmalı: aksi halde asperite enerjisi rezonatörlerde "çınlar" ve
tık kaybolur.

---

## 2. SÜRTÜNME / STICK-SLIP

### 2.1 Stick-slip neden bir *relaksasyon osilatörü*

İki yüzey, `k` yay sertliğiyle modellenen elastik bir bağla bağlı. Sürücü (el, ayak, menteşe) `v_drive` hızıyla
çekiyor; sürtünme kuvveti `μ·N`. Kuvvet `μ_s·N`'yi aştığı an yapışma kopar, kütle ileri fırlar, yay boşalır,
bağıl hız sıfıra düşünce yeniden yapışır. **Bu bir testere dişi osilatörüdür**; frekansı sürücü hızına bağlıdır:

```
d_slip  = μ_s · N / k              // kopma anındaki yay deplasmanı
f_naif  = v_drive / d_slip = v_drive · k / (μ_s · N)      [Hz]
```

**Ama `f_naif` yalnızca *yapışma* fazının hızıdır.** Gerçek olay sayısı, kayma fazının sonlu süresi
yüzünden daha düşüktür ve bir üst sınırla kesilir. §2.3'teki kodu 48 kHz'de çalıştırıp ölçtüm
(`k=20 kN/m, m=0.05 kg, c=0.2, μ_s·N=0.25`):

| v_drive | f_naif | **ölçülen f_ss** | oran | yorum |
|---|---|---|---|---|
| 2 mm/s | 160 Hz | **66.5 Hz** | 0.42 | hız sınırlı |
| 5 mm/s | 400 Hz | **90.0 Hz** | 0.22 | üst sınıra yaklaşıyor |
| 10 mm/s | 800 Hz | **98.0 Hz** | 0.12 | doyuyor |
| 20 mm/s | 1600 Hz | **100.5 Hz** | 0.06 | **doydu** |
| 50 mm/s | 4000 Hz | **101.0 Hz** | 0.03 | sürekli kayma |
| 100 mm/s | 8000 Hz | **101.0 Hz** | 0.01 | **creak yok, scrape** |

**Ölçülen yasa:**

```
f_ss  ≈ min( 0.42 · v_drive · k / (μ_s · N) ,  f_res )        f_res = (1/2π)·sqrt(k/m)
```

`f_res = 100 Hz` burada `sqrt(20000/0.05)/2π = 100.7 Hz` — yani **creak frekansı mekanik rezonansla
kesiliyor**. Bunun iki sonucu var:

1. **`k` keyfi seçilemez.** Hedef creak frekansı `f_c` ise `k ≥ m·(2π·f_c)²` olmalı.
   `m = 0.05 kg`, `f_c = 200 Hz` → `k ≥ 79 kN/m`. **01-dsp-sentez §2.7'deki `k = 200..5000 N/m`
   bu şartı sağlamıyor** (o değerlerle `f_res = 5..50 Hz` → ses-altı). Doğru aralık: **`k = 5·10⁴ … 5·10⁵`,
   `m = 0.01 … 0.1 kg`**.
2. **Regime geçişi kendiliğinden çıkıyor:** `v_drive` arttıkça olay sayısı `f_res`'te doyar; daha da artarsa
   kayma fazı yapışma fazından uzun olur ve ayrık "tık... tık" kaybolur → sürekli kayma (scrape).
   Bu, creak→scrape geçişinin fiziksel açıklaması; ayrı bir "mod" değil.

### 2.2 Stribeck eğrisi ve üç rejimin aynı bloktan çıkması

```
μ(v_rel) = μ_k + (μ_s - μ_k) · exp(-(v_rel/v_s)²)
```

`v_s` = Stribeck hızı (geçiş hızı). Eğrinin negatif eğimi **negatif sönümleme** üretir; sönüm yeterince
küçükse sistem Hopf bifurkasyonuyla kendi kendine salınır. En dik negatif eğim `v = v_s/√2`'de:

```
dμ/dv|max = -0.857 · (μ_s - μ_k) / v_s        [1/(m/s)]
kararsızlık:  N · |dμ/dv|  >  c               (c = yapısal sönüm)
salınım frekansı: f_sq = (1/2π)·sqrt(k/m)
```

**Ölçüm bunu doğruluyor** — squeal'de ölçülen olay frekansı, rezonans frekansıyla birebir aynı
(`k/m` değiştirildi, `v_drive` sabit):

| k | m | f_res = (1/2π)√(k/m) | **ölçülen f** | f_naif (v=100 mm/s) |
|---|---|---|---|---|
| 50 kN/m | 0.05 kg | 159 Hz | **159.0 Hz** | 2 857 Hz |
| 200 kN/m | 0.05 kg | 318 Hz | **318.5 Hz** | 11 429 Hz |
| 500 kN/m | 0.05 kg | 503 Hz | **503.5 Hz** | 28 571 Hz |

Yani squeal'de frekans **sürücü hızından bağımsız**, malzemenin mod frekansı. Creak'te ise tam tersi:
frekans `v_drive`'a bağlı (ta ki `f_res`'e doyana kadar). Bu ikisi aynı bloğun iki farklı çalışma noktası.

Üç rejim:

| Rejim | v_drive | k | μ_s-μ_k | v_s | Baskın frekans | Spektral karakter | Örnek |
|---|---|---|---|---|---|---|---|
| **Creak** (gıcırtı) | 1–20 mm/s | 5e4–5e5 | **0.25–0.45** | 1 mm/s | `min(0.42·v·k/(μ_s N), f_res)`, 20–400 Hz | harmonik değil, darbe treni + gövde | ahşap kasa, menteşe |
| **Squeal** (cıvıltı) | 50–500 mm/s | 2e5–1e6 | 0.05–0.15 | 20–100 mm/s | `f_res`, 700–4 000 Hz | neredeyse-sinus, dar bant | metal menteşe, fren, lastik |
| **Scrape** (sıyırma) | 0.1–2 m/s | (ilgisiz) | ~0 | — | broadband | sürekli gürültü + 30–500 Hz AM | sürüklenen sandalye, beton kazıma |

**Kritik ayrım:** creak'te `f_ss` *sürücü hızıyla değişir* (hızlanınca tizleşir, sonra doyar); squeal'de
`f_sq = f_res` *malzemenin mod frekansıdır* ve hızdan bağımsızdır (hız sadece genliği değiştirir).
Kod `rate_coupling ∈ {velocity, resonant}` olarak ikisini ayrı sunmalı.

### 2.3 Ayrık zaman güncellemesi ve kararlılık

```rust
#[derive(Clone, Copy, Debug)]
pub enum FrictionMode { Creak, Squeal, Scrape }

#[derive(Clone, Copy, Debug)]
pub struct FrictionParams {
    pub mode: FrictionMode,
    pub k: f32,        // 5e4 .. 5e5 N/m  (k >= m*(2*pi*f_creak)^2, bkz §2.1)
    pub mass: f32,     // 0.01 .. 0.1 kg
    pub damp: f32,     // 0.05 .. 5 N·s/m  (f_res'i sönümler; >5 creak'i boğar)
    pub load_n: f32,   // normal kuvvet, 0.2 .. 20 N
    pub mu_s: f32,     // 0.2 .. 0.9
    pub mu_k: f32,     // 0.1 .. 0.7   (mu_k < mu_s)
    pub v_s: f32,      // 0.0005 .. 0.1 m/s
    pub v_drive: f32,  // 0.001 .. 2 m/s
}

pub struct StickSlip {
    sr: f32, p: FrictionParams,
    sub: u32,                 // her örnekte alt-adım sayısı
    x: f32, y: f32, vy: f32,  // sürücü konumu, kütle konumu, kütle hızı
    stuck: bool,
    f_prev: f32,
    // çıkışlar
    pub force: f32,   // normalize F / (mu_s*N), ~0..1
    pub dfdt: f32,    // dF/dt — radyasyon için (patlama anı = broadband)
    pub slip_phase: f32, // 0..1, creak fazı (görselleştirme / grain tetiği)
}
```

Güncelleme (yarı-örtük / symplectic Euler, **kararlı**). Dikkat: bu blok ilk yazılışta ıraksadı —
iki tuzak §2.3.1'de.

```rust
impl StickSlip {
    pub fn new(sr: f32, p: FrictionParams) -> Self {
        // kararlılık: omega = sqrt(k/m); alt-adım, omega*dt <= 0.15 olacak şekilde
        let omega = (p.k / p.mass.max(1e-4)).sqrt();
        let sub = ((omega / (0.15 * sr)).ceil() as u32).clamp(1, 32);
        Self { sr, p, sub, x: 0.0, y: 0.0, vy: 0.0, stuck: true, f_prev: 0.0,
               force: 0.0, dfdt: 0.0, slip_phase: 0.0 }
    }

    #[inline]
    fn mu(&self, v_rel: f32) -> f32 {
        let r = v_rel / self.p.v_s.max(1e-5);
        self.p.mu_k + (self.p.mu_s - self.p.mu_k) * (-r * r).exp()
    }

    pub fn next(&mut self) -> f32 {
        let dt = 1.0 / (self.sr * self.sub as f32);
        let n_s = self.p.mu_s * self.p.load_n;
        let v_eps = self.p.v_drive.abs() * 0.02 + 1e-6;
        let mut f = 0.0f32;

        for _ in 0..self.sub {
            self.x += self.p.v_drive * dt;
            let z = self.x - self.y;                 // yay uzaması, + = sürücü ileride

            if self.stuck {
                f = self.p.k * z;                    // statik: sürtünme yayı dengeliyor
                if f.abs() > n_s { self.stuck = false; }
            }
            if !self.stuck {
                // sürtünme bloğun ZEMİNE göre hızına ters yönde — sürücü hızına değil
                let vrel = self.vy;
                let sgn = if vrel > 0.0 { 1.0 } else if vrel < 0.0 { -1.0 } else { 0.0 };
                let fric = -self.mu(vrel.abs()) * self.p.load_n * sgn;
                // m·ÿ = k·z (yay, + ileri) + f_fric - c·ẏ
                let acc = (self.p.k * z + fric - self.p.damp * self.vy) / self.p.mass.max(1e-4);
                self.vy += acc * dt;
                self.y += self.vy * dt;
                f = -fric;                           // iletilen (duyulan) temas kuvveti
                // yeniden yapışma: blok durdu VE yay hâlâ statik sınırın altında
                if self.vy.abs() < v_eps && (self.p.k * (self.x - self.y)).abs() < n_s {
                    self.stuck = true;
                    self.vy = 0.0;
                }
            }
        }

        let norm = 1.0 / n_s.max(1e-6);
        self.dfdt = (f - self.f_prev) * self.sr * self.sub as f32 * norm;
        self.f_prev = f;
        self.force = f * norm;
        self.slip_phase = (self.slip_phase + if self.stuck { 0.0 } else { dt * self.sr / 64.0 }).fract();
        self.force
    }
}
```

#### 2.3.1 İki uygulama tuzağı (ikisi de testte yakalandı)

**T1 — Sürtünmenin işareti.** İlk sürüm `v_rel = v_drive - vy` kullanıp sürtünmeyi `+μN·sgn(v_rel)`
olarak uyguluyordu. Sonuç: kütle zemin temasında değil, *sürücüye göre* sürtünüyor gibi davranıyor;
kuvvet bloğu hızlandırdığı yönde itiyor → `vy` ıraksıyor, 0.5 s içinde `NaN`.
Doğrusu: sürtünme **bloğun zemine göre hızına** (`vy`) terstir ve `m·ÿ = k·z + f_fric - c·ẏ`'de
`f_fric = -μ(|vy|)·N·sgn(vy)`. Sürücü hızı yalnızca yayın uzamasını (`x`) sürer.
> Konveyör bant modelinde (`x` = bandın konumu, kütle yaya bağlı) `v_rel = v_drive - vy` doğrudur.
> İki model karıştırılırsa sistem kararsız olur. Bizim fiziksel resim (el, menteşeyi çekiyor) yay-ucu modelidir.

**T2 — Yeniden yapışma koşulu.** Yalnızca `|vy| < v_eps` yetmez; blok durduğunda yay hâlâ statik sınırın
üstündeyse hemen tekrar kopar → Nyquist'te çığırtkanlık (`dfdt ≈ 96000`). Koşul **iki terimli** olmalı:
`|vy| < v_eps  &&  |k·(x-y)| < μ_s·N`.

**Ölçülen davranış (doğrulama):**

| Senaryo | Ölçülen | Beklenen | Not |
|---|---|---|---|
| Creak, v=2 mm/s | 66.5 Hz | 0.42·f_naif=67 Hz | ✓ |
| Creak, v=100 mm/s | 101.0 Hz | `f_res`=100.7 Hz | ✓ doyum |
| Squeal, k=50 kN/m | 159.0 Hz | `f_res`=159 Hz | ✓ hızdan bağımsız |
| Squeal, k=500 kN/m | 503.5 Hz | `f_res`=503 Hz | ✓ |
| Gravel, 120 ms, λ=5000/m, v tepe 0.3 m/s | yol 22.9 mm → **~115 tane** beklendi, 172 dolu örnek ölçüldü (taneler 1–3 örnek, örtüşmeli), peak 2.78 | ✓ ölçek doğru |

Squeal'de frekansın `f_res`'e birebir oturması, modelin Hopf bifurkasyonunu doğru yakaladığının kanıtı;
ayrı bir "squeal osilatörü" yazmaya gerek yok.

**Radyasyon seçimi — bu, sesin karakterini belirler:**

| Çıkış | Ne zaman | Spektrum |
|---|---|---|
| `force` (F) | gövde/panel titreşimi, düşük frekans | yumuşak, F süreklilik gösterir → az tık |
| `dfdt` (dF/dt) | havaya yayılan ses, temas "tık"ı | kopma anında **adım** → geniş bant; doğru seçim |
| `sgn(vy)` | "çakıl/kum" hissi | kare dalga → çok agresif, sadece scrape |

Doğru pratik: `out = 0.35·force + 0.65·dfdt` ve bu ikisini **modal banka ve HF click yoluna** paralel
gönder. Creak'te `dfdt` yolu ahşap modal bankına, `force` yolu düşük "gövde" filtresine.

### 2.4 Ucuz creak (ODE'siz) — 25 satır

Ses açısından `f_ss` ve jitter yeterli; ODE'nin detayı 2 kHz altında duyulmuyor. Bu sürüm %95 aynı sesi verir,
%10 CPU ile:

```rust
pub struct CheapCreak {
    sr: f32, phase: f32, rate_hz: f32,
    rate_jitter: f32, amp: f32, decay: f32,
    click_len: u32, click_pos: u32, click_amp: f32,
    rng: Rng,
}
impl CheapCreak {
    /// Kayma olayı = 1–3 örneklik işaretli impuls. Aralık Poisson.
    #[inline]
    pub fn next(&mut self) -> f32 {
        let mut out = 0.0;
        if self.click_pos < self.click_len {
            let t = self.click_pos as f32 / self.click_len as f32;
            out = self.click_amp * (1.0 - t);       // üçgen: -12 dB/oct, "-6 dB/oct tık" değil
            self.click_pos += 1;
        } else {
            self.phase += self.rate_hz * (1.0 + self.rng.normal() * self.rate_jitter) / self.sr;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
                self.click_len = 1 + (self.rng.next_f32() * 3.0) as u32;
                self.click_pos = 0;
                // genlik: Poisson dağılmış, %30 varyans — düzenli creak "böcek" gibi duyulur
                self.click_amp = self.amp * self.rng.range(0.55, 1.0);
                self.amp *= self.decay;                 // yavaş sönüm
            }
        }
        out
    }
}
```
`rate_hz = v_drive·k/(μ_s·N)`, `rate_jitter = 0.10..0.25`, `decay = 0.9995..0.99995` (örnek başına).

### 2.5 Granular sürtünme (çakıl / çamur / yaprak)

Ayak, çakıl yatağını `s` kadar sıkıştırırken **yol uzunluğu başına** taneler kırılır. Model:

```
λ = tane/metre (yüzey dokusuna bağlı)      N_beklenen = λ · s
her tane: Poisson zamanlı, lognormal genlik, 1–2 örnek genişlik, işaret rastgele
```

| Yüzey | tane boyu | λ (tane/m) | sıkışma | tane/adım | tane rezonansı |
|---|---|---|---|---|---|
| **Çakıl** (gravel) | 4–16 mm | 3 000–8 000 | 15–30 mm | 45–240 | 1.5–6 kHz |
| **Kum** (sand) | 0.1–1 mm | 40 000–120 000 | 3–8 mm | 120–960 | 6–16 kHz, çok kısa |
| **Toprak/çamur** | — | 5 000–20 000 | 10–25 mm | 50–500 | 400–1 500 Hz, sönümlü |
| **Kuru yaprak** | 20–80 mm | 100–400 | 5–15 mm | 1–6 | 3–9 kHz, uzun çınlama |
| **Cam kırığı** | 2–10 mm | 2 000–6 000 | 5–15 mm | 10–90 | 3–12 kHz, yüksek Q |

**Sürücü sinyali:** sabit hız değil, bir **hız zarfı** `v(t)` (ayak basışı: 0 → tepe → 0, 80–160 ms).
Tane üretimi `v(t)`'ye bağlı olduğu için ses doğal olarak "bastıkça artan, sonra kesilen" olur. Sabit hız
kullanmak — mevcut `Grain` katmanının yaptığı gibi — "shaker" hissi verir (01-dsp-sentez §2.6'da bu doğru
tespit edilmiş).

```rust
pub struct GranularFriction {
    sr: f32,
    pub density: f32,     // tane/metre
    pub roughness: f32,   // 0..1 genlik dağılımı
    path: f32,            // kat edilen yol (m)
    carry: f32,           // kesirli tane birikimi (deterministik!)
    v: f32,               // anlık sürücü hızı (m/s), dışarıdan set edilir
    pulses: [Pulse; 24],
    rng: Rng,
}

#[derive(Clone, Copy, Default)]
struct Pulse { pos: u32, len: u32, amp: f32, class: u8, dead: bool }

impl GranularFriction {
    #[inline] pub fn set_velocity(&mut self, v_mps: f32) { self.v = v_mps; }

    #[inline]
    pub fn next(&mut self) -> f32 {
        // 1) yol entegrasyonu → kesirli tane sayacı (kayıp yok, jitter yok)
        self.path += self.v.abs() / self.sr;
        self.carry += self.path * self.density;
        self.path = 0.0;
        while self.carry >= 1.0 {
            self.carry -= 1.0;
            self.spawn();
        }
        // 2) aktif taneleri topla
        let mut out = 0.0f32;
        let mut i = 0;
        while i < self.pulses.len() {
            let p = &mut self.pulses[i];
            if !p.dead {
                if p.pos < p.len {
                    let t = p.pos as f32 / p.len as f32;
                    out += p.amp * (1.0 - t) * (1.0 - t);   // kırılma: hızlı sönüm
                    p.pos += 1;
                } else { p.dead = true; }
            }
            i += 1;
        }
        out
    }

    fn spawn(&mut self) {
        let slot = match self.pulses.iter().position(|p| p.dead) { Some(s) => s, None => return };
        // lognormal, sigma=0.35, KIRPILMIŞ. u^-0.5 gibi sonsuz varyanslı dağılım kullanmayın:
        // tek bir u→0 örneği 1000× genlik üretip tüm sesi patlatır (testte oldu).
        let amp = (self.rng.normal() * 0.35).exp().clamp(0.15, 4.0);
        let p = &mut self.pulses[slot];
        p.pos = 0;
        p.len = 1 + (self.rng.next_f32() * 2.0) as u32;        // 1..3 örnek
        p.amp = amp * (1.0 + self.rng.normal() * 0.3 * self.roughness)
              * if self.rng.chance(0.5) { 1.0 } else { -1.0 };
        p.class = (self.rng.next_f32() * 4.0) as u8;           // 4 taş boyu sınıfı
        p.dead = false;
    }
}
```

**Tane sınıfı bankası (opsiyonel ama çok etkili):** 4 paralel 2-kutuplu rezonatör (2.2 / 3.4 / 5.1 / 7.8 kHz,
`T60 = 2–8 ms`); her impuls `class`'ına göre yönlendirilir. 4 mod × 5 FLOP = örnek başına 20 FLOP. Sonuç:
her tane *farklı boyutta bir taş* gibi duyulur. Tek bir BPF kullanmak tüm taneleri aynı yapar → "beyaz gürültü
+ AM" hissi, yani sentetik.

---

## 3. TRANSIENT / SPEKTRAL AKIŞ TASARIMI

### 3.1 Gerçek SFX'in attack'ı neden farklı

Gerçek bir darbe sesinin ilk 30 ms'inde üç şey *aynı anda* olur:

1. **Genlik:** 0.1–1 ms'de tepeye çıkar, sonra üstel düşer (τ = 0.5–15 ms).
2. **Spektral merkez:** 8–11 kHz'den 700 Hz'e **monoton** düşer. Düşüşün en dik yeri ilk 3 ms.
3. **Mikro-yapı:** 2–6 ayrı alt-olay vardır (temas, asperite kopması, yüzey kırılması, hava darbesi, yankı).
   Bunlar 0.2–20 ms aralıklarla, *örtüşerek* gelir.

Sentez attack'ları bu üçünü de kaçırır. Tipik hata sırası:

| # | Hata | Belirti | Düzeltme |
|---|---|---|---|
| H1 | Attack = "gürültü + sabit filtre" → spektrum attack boyunca **sabit** | centroid tek adımda düşer, "yapıştırılmış" hissi | filtre cutoff'u attack içinde de süpürülmeli (§3.5) |
| H2 | Attack zarfı **lineer rampa** (`env.rs perc`: 0.5 ms) | ilk 0.5 ms -6 dB ortalamayla kısılır; 1.2 ms crack'in %40'ı gider | raised-cosine 0.05–0.2 ms |
| H3 | Tek olay, mikro-yapı yok | "dijital click", "tek örnek tık" | 2–6 alt-olay, Poisson zamanlı |
| H4 | Düşük modlar `1/sinθ` ile şişmiş (§1.4) | boğuk + presence EQ ile telafi | `g_k = sinθ_k` |
| H5 | Tüm katmanlar aynı `t=0`'da, ama farklı filtre gecikmesiyle | faz uyumsuzluğu, transient dağılması | attack/gövde **sample-aligned**, filter pre-warp telafisi |

### 3.2 Sayısal hedefler (kategori tablosu)

Bu değerler **tasarım hedefidir**: ölçüm aralıklarından türetilmiş, preset yazarken referans olarak kullanılacak.
`SC` = spectral centroid (enerji-ağırlıklı). `crest50` = 50 ms pencerede tepe/RMS (dB).

| Ses | attack (ms) | tepe (ms) | crest50 | SC t≈0 | 3 ms | 20 ms | 100 ms |
|---|---|---|---|---|---|---|---|
| **Tabanca 9 mm** | 0.15–0.30 | 1.5–3 | 18–22 | 8–11 k | 5–7 k | 2.5–4 k | 700–1200 |
| **Tüfek 5.56** | 0.10–0.20 | 1–2 | 20–24 | 9–13 k | 6–8 k | 3–5 k | 600–1100 |
| **Pompalı 12 ga** | 0.30–0.60 | 3–6 | 15–19 | 4–7 k | 2.5–4 k | 1.2–2 k | 400–800 |
| **Susturucu** | 0.50–1.0 | 4–8 | 11–15 | 3–5 k | 1.5–2.5 k | 700–1200 | 250–500 |
| **Et isabeti** | 0.8–2.0 | 5–12 | 10–14 | 2.5–4 k | 1.5–2.5 k | 800–1400 | 300–600 |
| **Kemik kırılması** | 0.10–0.30 | 1–2 | 16–20 | 6–10 k | 4–6 k | 1.5–3 k | 500–900 |
| **Adım (beton)** | 0.50–1.2 | 3–8 | 10–14 | 3–6 k | 2–3.5 k | 800–1500 | 300–600 |
| **Kasa açılma** | 0.30–1.0 | 2–5 | 12–16 | 2.5–5 k | 1.5–3 k | 600–1200 | 200–500 |

**Tek en iyi "gerçek mi" testi:** `SC(0 ms) / SC(100 ms)` oranı. Gerçek darbe: **6–14×** (2.5–3.8 oktav).
Sentetik: 2–3× (1–1.5 oktav). Bu tek sayı, motorun durumunu ölçer.

### 3.3 Crest factor — zarfın doğrudan ölçüsü

`y = e^{-t/τ}` için 50 ms pencerede:

```
crest50_dB = 10·log10( 2·T / τ )  ,  T = 0.050 s       →  crest50 ≈ 10·log10(0.1/τ)
```

| τ | 0.3 ms | 1 ms | 2 ms | 5 ms | 10 ms | 20 ms |
|---|---|---|---|---|---|---|
| crest50 | 25.2 dB | 20.0 dB | 17.0 dB | 13.0 dB | 10.0 dB | 7.0 dB |
| crest20 | 21.2 | 16.0 | 13.0 | 9.0 | 6.0 | 3.0 |

Bu formül iki işe yarar: (a) hedef crest'ten τ seçmek, (b) render edilen buffer'ı QA'da ölçüp
"attack fazla yumuşak" uyarısı vermek. `τ < 1 ms` ise ve ölçülen crest 12 dB çıkıyorsa, araya giren
bir rampa/limiter attack'ı yemiş demektir.

### 3.4 Pre-emphasis ve transient shaper

**Pre-emphasis** — sadece **uyarım** yolunda, sesin tamamında değil. Birinci derece fark:

```
y[n] = x[n] - a·x[n-1]
+6 dB/oct köşesi: f0 = fs·(1-a) / (2π·sqrt(a))
a = 0.90 → f0 = 805 Hz      a = 0.95 → f0 = 393 Hz      a = 0.97 → f0 = 236 Hz
```

Bunu `ImpactExcitation` çıkışına uygulamak, "temas yüzeyi malzemeden daha sert" durumunu modeller
(çelik üstü ahşap, tırnak üstü masa). Modal bank girişinde `-6 dB/oct` bir tilt varsa (§1.4) bu
**tilt'i düzeltmenin en ucuz yolu** — ama doğrusu `g_k`'yi düzeltmek; pre-emphasis ondan sonra renk olarak kalır.

**Transient shaper** — attack'ı *ölçerek* güçlendirir, körlemesine EQ yapmaz:

```rust
pub struct TransientShaper {
    fast: f32, slow: f32,
    a_fast: f32, a_slow: f32,
    pub depth: f32,   // 0..2   (1.0 ≈ +6 dB attack)
}
impl TransientShaper {
    pub fn new(sr: f32, attack_ms: f32, release_ms: f32) -> Self {
        Self { fast: 0.0, slow: 0.0,
               a_fast: (-1.0 / (attack_ms  * 0.001 * sr)).exp(),
               a_slow: (-1.0 / (release_ms * 0.001 * sr)).exp(),
               depth: 0.0 }
    }
    #[inline] pub fn process(&mut self, x: f32) -> f32 {
        let a = x.abs();
        // attack: anında takip (max), release: üstel — klasik transient dedektörü
        self.fast = if a > self.fast { a } else { self.fast * self.a_fast + a * (1.0 - self.a_fast) };
        self.slow = self.slow * self.a_slow + a * (1.0 - self.a_slow);
        let d = (self.fast - self.slow) / (self.fast + 1e-6);
        x * (1.0 + self.depth * d.clamp(-1.0, 1.0))
    }
}
```
`attack_ms = 0.2`, `release_ms = 60`, `depth = 0.6..1.2`. Limiter'dan **önce**. Yalnızca ilk 5–20 ms'i
etkiler; gövde/kuyruk dokunulmaz. Bu blok, "sıradan bir darbe" ile "sinematik darbe" arasındaki en ucuz
farktır (±3 dB crest, ±1.5 LU algılanan yükseklik).

### 3.5 Attack / Body / Tail ayrıştırması — AAA silah reçetesi

Bu, sektörde fiilen kullanılan yapı; her katmanın **kendi transient'i, kendi filtresi, kendi stereo konumu**
vardır.

| Katman | İçerik | Süre | Filtre | Pan/delay | Gain |
|---|---|---|---|---|---|
| **1 ATTACK** | temas + asperite; **filtresiz**, tam bant | 0.3–2 ms | yok (HP 150 Hz sadece DC için) | mono, **0 ms** | 0…+3 dB |
| **2 CRACK** | patlama gövdesi, basınç dalgası | 15–60 ms | BP 500–4000, Q 0.6–1.2 | mono | −3…0 |
| **3 BODY** | 60–200 Hz basınç, pitch-drop | 40–150 ms | LP 300, Q 0.7 | mono, **0 ms** | −6…0 |
| **4 TAIL** | yansıma/oda | 150–900 ms | LP **düşen** 6 k→800 Hz | stereo, 2–15 ms pre-delay | −10…−16 |
| **5 MECH** | mekanizma, kovan, sürgü | 10–40 ms | modal bank | pan ±0.2, delay 15–120 ms | −18…−26 |
| **6 DEBRIS** | toz/moloz, Poisson grain | 100–600 ms | BP 1.5–4 kHz | stereo | −20…−30 |

**Değişmez kurallar:**

1. Katman 1–3 **0 ms gecikmeli** olmalı. 1 ms'lik bir kayma bile "iki ayrı ses" olarak duyulur
   (ekolojik işitme: 1 ms = 34 cm; aynı olaydan gelen sesler 0.5 ms içinde gelmeli).
2. Katman 1'in **hiçbir filtresi olmamalı**. Ona uygulanan her LP, sesi "uzaklaştırır".
3. `crest50` toplamda 18–22 dB olmalı. Katman 1 tek başına 24–28 dB crest taşır; gövde eklenince düşer.
4. Kuyruk **progresif olarak koyulaşmalı** — sabit LP "yapıştırılmış reverb" hissi verir.

**Progresif tilt uygulaması** (tek satır matematik, tek filtre):

```
fc(t) = fc0 · (fc1/fc0)^(t/T)          exponential, doğrusal DEĞİL
fc0 = 6000, fc1 = 800, T = 0.30 s
```
Mevcut altyapıda bu `FilterEnv { start_hz: 6000, end_hz: 800, tau: 0.10..0.15, q: 0.707 }` ile
*üstel* yaklaşımla yapılıyor — `FilterEnv::at` zaten `end + (start-end)·e^{-t/τ}`; `τ = T/2.3` seçilirse
`e^{-t/τ}` 0.1'e 0.3 s'de iner. Yani altyapı yeterli, **sadece preset'lerde kullanılmıyor**.

### 3.6 Mikro-yapı üretimi (H3'ün çözümü)

Attack'ı 2–6 alt-olaya bölmek, tek bir "click"ten kat kat daha gerçekçi:

| Alt-olay | Gecikme | Genlik | Süre | Örnek |
|---|---|---|---|---|
| **1. temas** | 0 | 1.0 | 0.5–1.5 ms | her zaman |
| **2. asperite kopması** | 0.3–3 ms | 0.3–0.7 | 0.1–0.4 ms | sert yüzey |
| **3. ikinci temas** | 2–12 ms | 0.1–0.4 | 0.3–2 ms | esnek cisim, yüzey düzensizliği |
| **4. rebound** | 8–40 ms | 0.05–0.25 | 1–5 ms | `e > 0.5` ise |
| **5. yüzey kırılması** | 0–30 ms | 0.1–0.5 | 5–40 ms | cam, kemik, tahta |
| **6. hava darbesi** | 0–2 ms | 0.2–0.6 | 5–30 ms | patlama, ateşli silah |

Zamanlama: Poisson (`-ln(u)/λ`), λ = 300–2000 olay/s → ilk 5 ms'de 2–8 olay. **Sabit aralık kullanmayın**,
tarak filtresi etkisi (comb) yapar.

---

## 4. NONLİNEERLİK VE "KİR"

### 4.1 Neden biraz nonlineerlik "gerçek" ve "yüksek" yapıyor

İki ayrı mekanizma var, karıştırılmamalı:

**a) Harmonik üretimi = kulak hassasiyet bandına enerji taşıma.**
Kulak 2–5 kHz'de en hassas (equal-loudness). 100 Hz'lik bir gövdeye `tanh` uygulamak 200/300/400 Hz üretir —
işe yaramaz, hatta gövdeyi "kirletir". Ama 800 Hz–3 kHz bölgesinde harmonik üretmek, algılanan yüksekliği
**peak'i artırmadan** 1.5–3 LU yükseltir. Bu yüzden doğru yer: **crack ve tail katmanları**, gövde değil.

**b) Crest factor sıkıştırma = "daha yüksek" hissi.**
`tanh` düşük seviyeleri `d` ile çarpar, tepeleri 1'e sıkıştırır → RMS/peak oranı artar. Loudness
RMS'e bağlı olduğu için aynı peak'te 2–4 dB daha yüksek duyulur. Bu, limiter'ın yaptığının
katman içinde, *harmonikleriyle birlikte* yapılmış hali.

### 4.2 Asimetrik şekillendirme — ve DC tuzağı

```
y = tanh( d·(x + b) ) - tanh(d·b)
```
`b` = asimetri (0.02–0.2), `d` = drive (1–8). İkinci harmonik oranı (küçük `d·b` için):

```
H2/H1 ≈ (d·b)/2        →  b=0.1, d=2  ⇒  -20 dB ;  b=0.2, d=4  ⇒  -14 dB
```

**DC tuzağı — mevcut kodda var.** `fx.rs:45` `((x + bias)*d).tanh() - bias.tanh()*0.5`.
`tanh(d·b)` çıkarmak doğru; `0.5·tanh(b)` çıkarmak **yanlış** (bias, drive uygulanmadan önce).
`asym=0.2, drive=1.41` için `x=0`'da çıkış **+0.064** — yani sessiz girişe +0.064 DC ekleniyor.
Master zincirindeki 2. DC blocker (38 Hz, τ=4.2 ms) bunu 4 ms'lik bir LF "thump"a çeviriyor.
Bu her sese eklenen, peak'i şişiren (normalizasyonu tetikleyen) ve duyulabilen bir hatadır.
**Doğrusu:** çıkışın ortalamasını *ölçerek* çıkarmak veya `b`'yi `x`'in işaretine bağlı uygulamak:

```rust
// DC'siz asimetri: bias'ı sinyalin kendi ölçeğine göre uygula, sonra ortalama-al
#[inline]
pub fn asym_shape(x: f32, d: f32, b: f32) -> f32 {
    let s = if x >= 0.0 { 1.0 + b } else { 1.0 - b };   // işaret-simetrik asimetri
    (x * s * d).tanh() / (1.0 + 0.35 * b)                 // kazanç kompanzasyonu
}
```
Bu, pozitif/negatif yarıdalga farkı üretir (H2) ama **DC üretmez** ve `tanh`'in simetrisini bozmaz.

### 4.3 Slew limiting — "dijital cızırtı"yı öldüren ucuz numara

```
y[n] = y[n-1] + clamp( x[n] - y[n-1], -S, +S )
```
`S` = örnek başına maksimum değişim. `S = 2π·f_slew/fs · A` (A = sinyal ölçeği).
`f_slew = 8 kHz`, `fs = 48k`, `A = 1` → `S = 1.047`.

Doğrusal fazlı bir LP değil ama **müzikal**: küçük sinyalleri geçirir, büyük ve hızlı değişimleri sınırlar →
tepe noktaları yumuşar, harmonik üretir, alias üretmez (kendisi bir LP). "Havayla taşınmış, uzaktan gelmiş"
hissi verir. **Kuyruk ve tail katmanında** kullanın; attack'ta kullanmayın (attack'ı öldürür).

Gerçek bir fiziksel karşılığı da var: hava absorbsiyonu + mikrofon/sürücü doygunluğu tam olarak
"yüksek frekansta sınırlı hız" davranışıdır.

### 4.4 Aliasing: ADAA (bedava anti-alias)

`tanh`'in ters-türevi `F(x) = ln cosh(x)`. Birinci derece ADAA:

```
y[n] = ( F(x[n]) - F(x[n-1]) ) / ( x[n] - x[n-1] )
```
Bu, `tanh`'in iki örnek arasındaki **ortalamasıdır** — yani doğal bir half-sample LP + nonlineerlik.
Aliasing'i 1× örneklemede 15–25 dB azaltır, 3 satır kod, sıfır bağımlılık:

```rust
#[inline]
fn ln_cosh(x: f32) -> f32 {
    let a = x.abs();
    a + (-2.0 * a).exp().ln_1p() - std::f32::consts::LN_2
}

pub struct AdaaTanh { x1: f32, d: f32 }
impl AdaaTanh {
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let xs = x * self.d;
        let f1 = ln_cosh(xs);
        let f0 = ln_cosh(self.x1 * self.d);
        let dx = (x - self.x1) * self.d;
        self.x1 = x;
        if dx.abs() < 1e-4 { xs.tanh() } else { (f1 - f0) / dx }
    }
}
```

### 4.5 Hangi aşamada kaç × oversampling

| Aşama | İçerik bandı | Gerekli OS | Gerekçe |
|---|---|---|---|
| **Asperite / attack shaper** | 10 kHz+ | **4×** | en yüksek harmonik yoğunluğu; 1× %100 alias |
| Crack shaper (BP 500–4 k) | 4 kHz | **2×** | 3. harmonik 12 kHz'de kalır, güvenli |
| Tail / kuyruk saturasyonu | 2 kHz (LP'li) | **1×** (ADAA) | ADAA yeterli |
| Body / sub (60–200 Hz) | 200 Hz | **1×** | harmonikler 2 kHz altı; OS israf |
| Master drive | tam bant | **4×** | tüm katmanların harmonikleri üst üste biner |
| Bitcrush / decimate | bilinçli alias | **1×** | alias *amaç* |
| Slew limiter | — | **1×** | kendisi LP |

**2×/4× oversampling, kütüphanesiz** (offline render'da bedava):

```rust
/// Halfband FIR'ı prepare()'da üret (Blackman-windowed sinc, 31 tap).
/// Merkez tap = 0.5, çift indeksler ≈ 0 (halfband özelliği).
fn halfband(taps: usize) -> Vec<f32> {
    let m = (taps / 2) as f32;
    (0..taps).map(|i| {
        let n = i as f32 - m;
        let sinc = if n.abs() < 1e-6 { 1.0 } else { (std::f32::consts::PI * n * 0.5).sin() / (std::f32::consts::PI * n * 0.5) };
        let w = 0.42 - 0.5 * (std::f32::consts::TAU * i as f32 / (taps - 1) as f32).cos()
                    + 0.08 * (2.0 * std::f32::consts::TAU * i as f32 / (taps - 1) as f32).cos();
        sinc * w
    }).collect()
}
```
Yukarı örnekleme: sıfır ekle → FIR → ×2. Aşağı: FIR → her 2. örneği al → ×2.
`prepare()`'da bir kez üretilir, `render` sırasında allocation yok.

### 4.6 Nereye **koymamalı**

| Yasak | Neden | Sonuç |
|---|---|---|
| DC blocker'dan **önce** asimetrik shaper | bias → DC adımı | 4–10 ms LF thump, woofer gezintisi, headroom kaybı |
| **Tüm mix** üzerinde `tanh` | katmanlar arası intermodülasyon | transient tanımı yok olur, "duvar ses" |
| **Sub / body** katmanında drive | 100 Hz'in harmonikleri 300–500 Hz'de gövdeyle çakışır | "kirli", tanımsız alt uç |
| Modal bank **çıkışında** shaper | rezonatörler harmonikleri çınlatır | malzeme kimliği bozulur (bazen istenen "dirt") |
| 0.4·fs üstü içerik + 1× shaping | alias | "cızırtı", özellikle yüksek drive'da |
| Limiter'dan **sonra** drive | ceiling aşılır | clip + normalizasyon çelişkisi |

---

## 5. MEVCUT MOTORDAN NE SİLİNMELİ / YENİDEN AYARLANMALI

Kod okundu; aşağıdakiler **somut, dosya/satır** ve gerekçeli. `(K)` = kritik, `(Y)` = yüksek, `(O)` = orta.

| # | Yer | Sorun | Neden sentetik | Aksiyon |
|---|---|---|---|---|
| **1 (K)** | `render.rs:208-221` `LayerKind::Modal` exciter | Gürültü × **lineer rampa** (HP→LP) | Kuvvet pulsunun şekli yok; `Tc` yok; spektrum "psshh" | `ImpactExcitation` ile değiştir (§1.7). `exciter_ms` → `tc` |
| **2 (K)** | `modal.rs:59` `y = a1·y1 + a2·y2 + x` | `g = 1` → örtük `1/sinθ` | Her bank **-6 dB/oct** yanlış tilt; malzeme kimliği `amp`'ta değil `f`'te | `g_k = sinθ_k` (impuls) / `sqrt(1-r²)` (gürültü). Uygulama: 02 §3.2 |
| **3 (K)** | `fx.rs:42-48` `Saturator` asym | `- bias.tanh()*0.5` DC'yi iptal etmiyor | Her sese +0.064 DC → 4 ms thump; peak şişer | `asym_shape` (§4.2) veya `bias=0` |
| **4 (K)** | `env.rs:28-35` `AdEnv::perc` | `attack = 0.5 ms`, `attack_curve = 1.0` (lineer) | 1.2 ms crack'in %40'ı gider; türev süreksizliği "fff" ekler | `attack = 0.05..0.2 ms`, raised-cosine (`x²(3-2x)`) |
| **5 (Y)** | `render.rs:193` `decay_scale = 1 + style*0.65` | Cartoon'da rezonans **uzuyor** | 01-dsp §4.2/D2 `modal_Q ×0.25` diyor — **çelişki** | `1.0 - 0.55*style` (kısalma) |
| **6 (Y)** | `render.rs:153` `end = end_hz + (start-end)*style*0.45` | Cartoon'da pitch düşüşü **azalıyor** (165→78 yerine 165→117) | "Boing" yerine "daha az boing" | `end = end_hz - (start-end)*0.55*style` |
| **7 (Y)** | `render.rs:350-356` bitcrush (`style>0.6`, 512→128) | Her katmana, decimation yok, OS yok | Tüm "cartoon" seslerde alias cızırtısı | **Sil**. Yerine 12 sürücülü stil haritası |
| **8 (Y)** | `render.rs:420-428` reverb `(l+r)*0.5` | Mono send; sonra `widen` (L=R'de etkisiz) | Her kuyruk **tam merkezde** = "küçük/sentetik mekân" | Stereo dekorele send (2 bağımsız IR / farklı comb seti) |
| **9 (Y)** | `noise.rs:84-90` `Velvet` | Örnek-başı Bernoulli, ızgara değil | Yoğunluk semantiği yanlış (0.01 = 480 pps); seviye normalize değil | `Td = fs/density` ızgara, hücre başına 1 puls, `1/sqrt(n)` |
| **10 (Y)** | `noise.rs:76-79` `Brown` | `clamp(-1,1)` sonra `×3.5` | Integratör kırpılıyor → distorsiyon + DC; HP yok | Sızdıran integratör + 20 Hz HP |
| **11 (Y)** | `fx.rs:254-295` `Limiter` | `attack = 0.8 ms`, lookahead yok | Crack'in ilk 0.8 ms'i sınırsız geçer, sonra normalize her şeyi kısar | 1.5–2 ms lookahead + gain ramp |
| **12 (Y)** | `fx.rs:250` `(l*0.09, r*0.09)` | Sabit kazanç; comb `feedback 0.72..0.98` | `reverb_size` hem süre hem **seviye** kontrolü → preset tuning bozulur | `feedback`'ten bağımsız çıkış normalize: `×(1-fb)/sqrt(8)` |
| **13 (O)** | `fx.rs:116-147` `Comb` | Girişte HP yok | DC birikir, kuyruk "şişer" (01-dsp §2.9 uyarısı) | Comb içine 20 Hz HP |
| **14 (O)** | `render.rs:410` `comp.set_makeup_db(2.0)` | Sabit | `ratio`/`threshold` değişince seviye zıplar | `makeup = (1/ratio - 1)·threshold_db·0.5` |
| **15 (O)** | `render.rs:396-406` master drive | Tüm mix, EQ'dan sonra, comp'tan önce | Katmanlar arası IM; transient düzleşir | Katman başına, attack hariç |
| **16 (O)** | `modal.rs:45` `clamp(0.0, 0.99999)` | `T60` ~14 s'de kırpılır; `f < 150 Hz`'de f32 `cos` hata | Uzun çan modları yanlış söner | f64 katsayı (f<200 Hz), clamp `0.999999` |
| **17 (O)** | `modal.rs:72` `Vec<ModeState>` AoS + `render.rs:210` tam buffer döngüsü | SIMD yok, erken çıkış yok | (CPU) 48 mod × tam buffer = boşa iş | SoA + `is_silent` erken çıkış (02 §3.2) |
| **18 (O)** | `crates.rs:93` "stick slip" katmanı | Gerçekten `Saw` + 5.5 Hz `vibrato` | Sürtünme değil, LFO'lu testere | `StickSlip` (§2.3) + `CheapCreak` (§2.4) |
| **19 (O)** | `foley.rs` adım presetleri | 7 ms gürültü + modal ring + sine | §2.5 granular sürtünme yok → "tık + bum" | `GranularFriction` + §1.7 temas |
| **20 (O)** | `render.rs:357` `let _ = soft_clip;` | Ölü kod; `soft_clip` `tanh` değil (`x=1`'de 0.777 vs 0.762) | — | Kullan veya sil |
| **21 (O)** | `osc.rs:99` `PitchEnv` `t += 1/fs` | f32 akümülasyon; `EnvGen` örnek sayacı kullanıyor (tutarsız) | Uzun kuyrukta ms mertebesi kayma | Sayaç (`n as f32 / sr`) |
| **22 (O)** | `osc.rs:46` `Triangle` | Band-limitli değil, türev süreksiz | 1 kHz üstünde alias | Sadece <300 Hz veya PolyBLEP |

### 5.1 `LayerKind` sözleşme değişikliği (öneri)

```rust
// ESKİ
Modal { modes: Vec<Mode>, exciter_ms: f32, exciter_color: NoiseColor,
        exciter_hp: f32, exciter_lp: f32, detune_cents: f32 }

// YENİ
Impact {
    modes: Vec<Mode>,
    contact: ImpactParams,     // mass, radius, e_star, restitution, hardness, asperity
    velocity: f32,             // 0..1 — ZORUNLU, artık opsiyonel değil
    strike_pos: f32,           // 0..1, g_k *= |sin(k π x)| (01-dsp §2.3)
}
Friction {
    modes: Vec<Mode>,          // gövde rezonatörleri
    params: FrictionParams,    // mode, k, mass, mu_s/k, v_s, v_drive
    drive: DriveShape,         // Constant | Envelope(AdEnv) | VelocityTrack(f32)
}
Granular { params: GranularParams, classes: [f32; 4] /* taş boyu rezonansları */ }
```

`exciter_hp`, `exciter_lp` **kalkar**: bunlar `f_-3dB = 0.634/Tc` ve `f_tilt`'ten otomatik türetilir.
Elle girilen bu iki değer, preset'lerin birbirine benzemesinin başlıca sebebi (hepsi 300–600 / 11–13 k).

---

## 6. QA: ölçülebilir kabul kriterleri

FFT yok, bağımlılık yok. `f_rms` = spektral RMS frekansı (ikinci moment; centroid'den ~1.2–1.6× yüksek
ama monoton — A/B için mükemmel):

```rust
/// f_rms ≈ fs/(2π) · sqrt( Σ(x[n]-x[n-1])² / Σx[n]² )
/// Tek sinüsoid için TAM olarak f verir. 20 satır, FFT yok.
pub fn spectral_rms_hz(x: &[f32], sr: f32) -> f32 {
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for i in 1..x.len() {
        let d = (x[i] - x[i - 1]) as f64;
        num += d * d;
        den += (x[i] as f64) * (x[i] as f64);
    }
    if den < 1e-12 { return 0.0; }
    (num / den).sqrt() as f32 * sr / std::f32::consts::TAU
}
```

| Kriter | Eşik | Nasıl |
|---|---|---|
| **Spektral düşüş** | `f_rms(0..2ms) / f_rms(80..100ms) ≥ 4` | iki pencere, §6 fonksiyonu |
| **Crest** | `crest50 ∈ [12, 24] dB` (kategoriye göre §3.2) | `peak/ rms(50ms)` |
| **Attack** | ilk %10 tepe örneği ≤ 0.3 ms'de | tepe indeksi |
| **DC** | `|mean| < 0.002` | `AudioBuffer::dc_offset` |
| **Aliasing** | 20 kHz üstü enerji < −60 dBFS | 4× upsample + LP + ölç (gerçek alias testi) |
| **Mono uyum** | `corr(L,R) > -0.2` | mevcut `widen` çıkışında |
| **Varyans** | 8 varyasyon arası `f_rms` std < %15, süre std < %20 | 01-dsp §5.3 |
| **Determinizm** | aynı seed → **bit-identical** | iki render, `==` |

---

## 7. Sabitler ve formül indeksi

```
Hertz temas:        k = (4/3) E* sqrt(R)
                    Tc = 2.94 (5m/(4k))^0.4 v^-0.2
                    δmax = (5 m v²/(4k))^0.4
Kuvvet spektrumu:   f_-3dB = 0.634/Tc   f_-10dB = 1.09/Tc   f_-20dB = 1.41/Tc
Momentum:           J = m v (1+e)
Modal kazanç:       impuls: g_k = sin(2π f_k/fs)      gürültü: g_k = sqrt(1-r_k²)
                    r_k = exp(-6.907755/(T60_k fs))
Stick-slip:         d_slip = μ_s N / k
                    f_naif = v_drive k /(μ_s N)
                    f_ss   = min(0.42 f_naif, f_res)          (ÖLÇÜLDÜ, §2.1)
                    f_res  = (1/2π) sqrt(k/m)                 (creak doyum + squeal frekansı)
                    k ≥ m (2π f_creak)²                       (tasarım şartı)
Stribeck:           μ(v) = μ_k + (μ_s-μ_k) exp(-(v/v_s)²)
Negatif eğim:       max |dμ/dv| = 0.857 (μ_s-μ_k)/v_s   @ v = v_s/√2
Kararsızlık:        N |dμ/dv| > c
Sürtünme işareti:   f_fric = -μ(|vy|) N sgn(vy)           (vy = bloğun ZEMİNE göre hızı)
Crest:              crest50_dB = 10 log10(0.1/τ)          [τ saniye]
Pre-emphasis:       y = x - a x[n-1] ;  f0 = fs(1-a)/(2π√a)
Slew:               S = 2π f_slew/fs · A
ADAA:               y = (ln cosh(x[n]) - ln cosh(x[n-1]))/(x[n]-x[n-1])
Granular:           N = λ · s      λ = tane/metre
Spectral RMS:       f_rms = fs/(2π) sqrt(Σ(Δx)²/Σx²)
Poisson zamanlama:  Δt = -ln(u)/λ  (ASLA sabit aralık)
```

---

## 8. Uygulama sırası (bağımlılık sıralı)

1. `modal.rs` `g_k = sinθ` normalizasyonu + presetlerdeki `exciter_*` / `presence` / `drive` değerlerini
   **birlikte** yeniden tune et (tilt düzelince EQ'lar fazla parlak olur). → 1 gün, en yüksek etki.
2. `excite.rs`: `ImpactExcitation` + `LayerKind::Impact`. → 2 gün.
3. `env.rs` attack düzeltmesi + `TransientShaper` (master zincirin başına). → 0.5 gün.
4. `fx.rs` `Saturator` DC hatası + master → katman taşıması. → 0.5 gün.
5. `excite.rs`: `StickSlip` + `GranularFriction` + `LayerKind::Friction/Granular`. → 2 gün.
6. Reverb stereo send + limiter lookahead. → 1 gün.
7. §6 QA fonksiyonları + `sg-engine` test eşikleri. → 1 gün.

**Toplam ~8 gün.** 1–4 arası tek başına "sentetik" şikâyetinin %70'ini çözer.
