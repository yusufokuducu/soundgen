# 01 — DSP ve Sentez Araştırması

> **Kapsam:** SoundGen'in ses üretim çekirdeği. Hangi primitive'ler, hangi matematik, hangi parametre aralıkları,
> hangi kategori için hangi katman grafiği, gerçekçilik↔stil morph ekseni, varyasyon stratejisi, loudness,
> performans bütçesi ve uygulama tuzakları.
> **Hedef okuyucu:** yarın C++/Rust yazmaya başlayacak DSP mühendisi.
> Tüm sayısal değerler **48 kHz** örnekleme hızı için verilmiştir; `fs` sembolü örnekleme hızıdır.

---

## 0. Yönetici Özeti (karar verilmiş varsayımlar)

1. **Motor bir DAG (directed acyclic graph) renderer'dır**, sample-bazlı değil **block-bazlı** çalışır (128 veya 256 örnek),
   ama olaylar (`trigger`, `release`, `param breakpoint`) **sample-accurate** zamanlanır. Her node `process(ctx, out, n)` yazar;
   kontrol parametreleri block içinde örnek-örnek yumuşatılır.
2. **Her efekt = katman yığını.** Katmanlar bağımsız render edilip toplanır; her katmanın kendi seed'i, kendi gain/pan/filter/IR send'i vardır.
   Bu, "aynı sesin 12 kardeş varyasyonu" üretmenin tek ölçeklenebilir yolu.
3. **Modal synthesis çekirdek çalgı motorudur** (ahşap, metal, plastik, kemik, beton, cam). Geri kalan her şey
   noise + transient + filter + granular + physical model katmanlarıdır.
4. **Gerçekçilik↔stil ekseni tek bir `style ∈ [0,1]` parametresidir** ve *her kategoride* ayrı bir eşlem tablosuyla
   somut DSP parametrelerine bağlanır (bkz. §4). Global bir "cartoon" makrosu değil, kategori-duyarlı bir vektördür.
5. **Deterministik render zorunlu:** aynı `seed` + aynı parametre vektörü = bit-identical çıktı. Bu, offline render,
   A/B dinleme ve oyun içi "family" üretimi için ön koşul.

---

## 1. Render Mimarisi

### 1.1 Node sözleşmesi

```
struct ProcessContext {
    double sample_rate;      // 48000
    int    block_size;       // 128
    uint64_t frame_index;    // global, deterministic
    float* out_l; float* out_r;   // block buffers, 32-byte aligned
    const EventList* events;      // sample-accurate, sorted
};
```

Kurallar:

- **Audio thread'de allocation yok, lock yok, syscall yok.** Tüm voice'lar `prepare()` aşamasında preallocate edilir.
- Her node `prepare(fs, max_block)` ve `reset()` uygular; sample-rate'e bağlı sabitler **sadece** `prepare`'da hesaplanır.
- Parametre yumuşatma: tek kutuplu (`smooth_coef = exp(-1/(tau*fs))`, `tau = 5..20 ms`) veya block-içi lineer rampa.
  Cutoff/pitch gibi log-domain parametreler **log uzayında** yumuşatılır (zipper ve "portamento hissi" için).
- Render **iki fazlı**: `render_offline()` (worker thread, gerçek zamandan bağımsız, daha büyük block) ve
  `render_realtime()` (audio callback). Aynı DSP kodu, farklı block boyu. Audition her zaman offline render + ring buffer playback'tir;
  böylece "tuşa basınca çalsın" ile "stabil DSP" çelişmez.

### 1.2 Katman tipleri (node kataloğu)

| Node | Amaç | Kritik parametreler |
|---|---|---|
| `NoiseGen` | white/pink/brown/velvet kaynak | `type`, `density`, `seed` |
| `TransientGen` | 0.2–10 ms darbe/click | `duration`, `curve`, `polarity` |
| `ModalBank` | 8–64 decay eden resonator | `f[]`, `Q[]` veya `T60[]`, `gain[]` |
| `Svf` / `Biquad` | LP/BP/HP/notch/peak | `fc`, `Q`, `mode` |
| `FormantBank` | 3–5 paralel BPF | `F1..F5`, `BW`, `gain` |
| `Osc` | sine/saw/square/wavetable | `f0`, `shape`, `phase` |
| `PitchEnv` | f(t) eğrisi | `start_ratio`, `end_ratio`, `tau` |
| `FmPair` | PM/FM | `ratio`, `index`, `env` |
| `Shaper` | tanh/foldback/bitcrush | `drive`, `bits`, `decim` |
| `GrainCloud` | granüler doku | `grain_ms`, `density`, `spray`, `rate` |
| `KsString` / `Waveguide` | telli/boru | `f0`, `damping`, `brightness` |
| `StickSlip` | gıcırtı/friction | `v_slip`, `stiffness`, `mu` |
| `Blast` | Friedlander + combustion | `Ps`, `td`, `b` |
| `Rotor` | blade-pass AM + doppler | `rpm`, `blades`, `depth`, `drift` |
| `Convolver` | kısa IR (gövde/kuyruk) | `ir`, `wet`, `pre_delay` |
| `Reverb` | Freeverb/Schroeder | `room`, `damp`, `wet` |
| `Eq`, `Comp`, `Limiter`, `Pan`, `Gain` | mix utility | — |

---

## 2. Sentez Primitive'leri

### 2.1 Gürültü üreticileri

Beyaz gürültü **tek başına** kullanılmaz; her zaman filtrelenir. Ama filtrenin girişindeki spektrum, çıkışın karakterini belirler.

| Tip | Üretim | Spektral eğim | Kullanım |
|---|---|---|---|
| White | `x = 2*u01 - 1` (PCG32 / xoshiro128**) | 0 dB/oct | transient, click, HF crack |
| Pink | Kellet 6-kutuplu filtresi | −3.01 dB/oct | rüzgâr, kalabalık, "air", foley |
| Brown/red | sızdıran integratör | −6 dB/oct | motor gövdesi, uzak patlama, deprem |
| Velvet | seyrek ±1 darbe | düz (1500–2000 darbe/s üzeri) | reverb kuyruğu, kâğıt hışırtısı |
| Blue | beyazın türevi | +6 dB/oct | parlak "tsss", silah crack'i (destekleyici) |

**Pink (Paul Kellet, 6 kutuplu "refined" versiyon — 100 Hz–20 kHz arası ±0.2 dB):**

```
b0 = 0.99886*b0 + w*0.0555179;
b1 = 0.99332*b1 + w*0.0750759;
b2 = 0.96900*b2 + w*0.1538520;
b3 = 0.86650*b3 + w*0.3104856;
b4 = 0.55000*b4 + w*0.5329522;
b5 = -0.7616*b5 - w*0.0168980;
pink = b0+b1+b2+b3+b4+b5+b6 + w*0.5362;
b6 = w*0.115926;
```

**Brown:** `y = (y + 0.02*w) * 0.995;` + zorunlu 20 Hz high-pass (DC drift'i engeller). Integratör kazancı
`0.02` seçilmezse seviye `fs`'e bağımlı olur — bu klasik hata.

**Velvet noise:** periyodu `Td = fs/density` olan ızgarada, her hücrede rastgele konumda, rastgele işaretli tek darbe.
`density = 1500..2000 pulse/s` altında kulağa "pürüzlü/rough" gelir; üstünde düz bant gürültüsü gibi duyulur
(Välimäki ve ark.'nın velvet-noise literatüründe bu eşik açıkça raporlanmıştır). SoundGen'de reverb kuyruğu ve
"paper rustle" için varsayılan `1800 pulse/s`.

**Noise shaping:** yalnızca bit-depth düşürmede (16-bit export) veya bilinçli "lo-fi" efektinde. Hata geri beslemesi:
`e[n] = q(x[n] + a*e[n-1]) - (x[n] + a*e[n-1])`, `a = 0.5..0.9`. Oyun döngülerinde **kullanılmaz** (yorgunluk yapar).

**Seed:** tek bir `uint64 seed` → SplitMix64 ile `(layer_id, param_id)` hash'lenerek alt-seed üretilir. Böylece
bir katmanı değiştirmek diğer katmanların gürültüsünü yeniden karıştırmaz.

### 2.2 Transient ve envelope

Bir darbe (impact) sesinin "vuruculuğu" iki şeyden gelir: **0.2–3 ms yükselme** ve **ilk 10 ms'deki spektral merkez**.
Daha yavaş attack = "yumuşak/uzak", daha hızlı = "yakın/sert".

| Envelope | Formül | Kullanım |
|---|---|---|
| Exponential (percussive) | `y = A*exp(-t/tau)`, `tau = T60/6.9078` | modal, kuyruk, genel decay |
| RC attack+decay | `y = A*(1-exp(-t/tau_a))*exp(-t/tau_d)` | "analog" yumuşak vuruş |
| Power curve | `y = A*(1-t/T)^gamma`, `gamma = 2..6` | "snappy" click, UI |
| Linear AR | `y = A*min(t/t_a,1)*max(0,1-t/T)` | nötr, dijital hissiyat |
| Two-stage | hızlı exp + yavaş exp toplamı | silah kuyruğu, gövde |

**T60 → tau:** `tau = T60 / ln(1000) = T60 / 6.907755`.
**T60 → kutup yarıçapı:** `r = 10^(-3/(T60*fs)) = exp(-6.907755/(T60*fs))`.

Referans değerler (48 kHz):

| T60 | r | eşdeğer −3 dB bandwidth |
|---|---|---|
| 50 ms | 0.997126 | 44.0 Hz |
| 100 ms | 0.998562 | 22.0 Hz |
| 300 ms | 0.999520 | 7.3 Hz |
| 1 s | 0.999856 | 2.2 Hz |
| 2 s | 0.999928 | 1.1 Hz |

**Kritik:** envelope'un sonuna **asla** sert kesme koymayın; son 5–20 ms'de raised-cosine fade uygulanır, aksi halde
`click` duyulur. Aynı şey envelope'un **başında** da geçerli: 0.2 ms'den kısa attack'ta bile en az 8 örnek rampa.

**Polarity:** patlama/ateş seslerinde ilk yarım dalganın pozitif olması "punch" algısını değiştirir; `polarity` parametresi
olarak maruz bırakılmalı (özellikle kick-benzeri gövde katmanlarında).

### 2.3 Modal synthesis (çekirdek)

Sürekli sistemin modal açılımı, ayrık zamanda **ikinci dereceden paralel resonator bankası**dır:

```
H_k(z) = g_k / (1 - 2*r_k*cos(theta_k)*z^-1 + r_k^2*z^-2)
theta_k = 2*pi*f_k/fs
r_k     = exp(-pi*BW_k/fs),  BW_k = f_k/Q_k   veya   r_k = 10^(-3/(T60_k*fs))
```

Zaman tanımında:

```
y_k[n] = a1_k*y_k[n-1] + a2_k*y_k[n-2] + g_k*x[n]
a1_k = 2*r_k*cos(theta_k)
a2_k = -r_k^2
```

**Kazanç normalizasyonu.** Eşit enerji için `g_k = sqrt(1 - r_k^2)`; malzemenin spektral eğimi için
`g_k *= (f_k/f_0)^(-beta/2)`, `beta = 0..2` (0 = parlak metal, 2 = boğuk ahşap).
Toplam çıkışı ölçerek normalize etmek en güvenilir yoldur: `prepare()` aşamasında her modun ilk 4*tau'luk impulse
response'unun RMS'i hesaplanır, `sum_k g_k^2 * E_k = 1` olacak şekilde ölçeklenir.

**Mod oranları (f_k/f_1) — malzeme kimliği buradadır:**

| Nesne | Oranlar | Not |
|---|---|---|
| İdeal string | 1, 2, 3, 4, 5, 6 | harmonik; sadece "telli" şeyler |
| Serbest-serbest bar (metalo­fon, çubuk) | 1, 2.756, 5.404, 8.933, 13.34 | gerçek metal çubuk |
| Dairesel plaka (free edge) | 1, 1.594, 2.136, 2.296, 2.653, 2.918, 3.156 | zil/tepsi/trap |
| Dikdörtgen membran | 1, 1.414, 2, 2.236, 2.828, 3, 3.162 | davul derisi |
| Ahşap kutu (non-uniform) | 1, 2.32, 3.71, 5.14, 6.90 | kasa, sandık, tahta |
| Çan | 0.5, 1.0, 1.19, 1.5, 2.0, 2.5, 3.0 | hum/prime/tierce |
| Cam | 1, 2.7, 4.1, 6.3, 8.9 | yüksek Q, hızlı HF decay |

**Damping.** Gerçek nesnelerde yüksek modlar daha hızlı söner:
`T60_k = T60_0 * (f_k/f_0)^(-alpha)`, `alpha = 0.2..0.7`. `alpha=0` yapay/organdır; `alpha=0.5` ahşap, `0.7` cam.
Q tablosu (f=1 kHz referans): ahşap 15–60, plastik 30–100, beton 20–80, metal 200–1500, cam 500–3000, et 2–8.

**Mistuning zorunlu.** Her moda `f_k *= 1 + N(0, 0.005..0.02)` uygulanır. Sıfır mistuning "synthesizer" gibi duyulur;
%1–2 arası "gerçek nesne" hissi verir. Stil ekseninde bu değer %4–6'ya kadar açılır (oyuncak/cartoon).

**Excitation pozisyonu.** String/plate benzeri nesnelerde `g_k *= |sin(k*pi*x)|`, `x = 0.08..0.35` (vuruş noktası).
Bu tek parametre, aynı modal banktan "kenardan vurulmuş" ve "ortadan vurulmuş" iki farklı ses üretir.

**Maliyet:** mod başına 3 çarpma + 2 toplama (≈5 FLOP). 48 modlu bir bank, 48 kHz'de ≈11.5 MFLOP/s; AVX2 ile 8 mod paralel.

### 2.4 Subtractive filtreler

**Tercih: TPT (topology-preserving transform) state-variable filter.** Chamberlin SVF yüksek cutoff'ta ve hızlı
modülasyonda kararsızdır; TPT zero-delay-feedback olduğu için `fc → fs/2` sınırında bile kararlıdır ve modülasyon
sırasında "patlamaz".

```
g  = tan(pi*fc/fs)          // prewarped
k  = 1/Q                    // damping
a1 = 1/(1 + g*(g+k))
a2 = g*a1
a3 = g*a2

v3 = x - ic2eq
v1 = a1*ic1eq + a2*v3
v2 = ic2eq + a2*ic1eq + a3*v3
ic1eq = 2*v1 - ic1eq
ic2eq = 2*v2 - ic2eq

lp = v2
bp = v1
hp = x - k*v1 - v2
notch = x - k*v1          // = hp + lp
```

`fc` her örnekte güncellenebilir. **Yavaş modülasyonda** (cutoff sweep, envelope) `g`'yi her 8–16 örnekte bir yeniden
hesaplamak CPU'dan %60–70 tasarruf eder ve duyulabilir fark yaratmaz (16 örnek = 0.33 ms). Hızlı/audio-rate
modülasyonda (LFO > 20 Hz, FM) ise her örnekte güncellenmelidir; ara Çözüm: 8 örnekte bir güncelle + katsayıları
lineer interpole et.

**RBJ biquad** sadece statik EQ için (reverb damping, ton kontrolü). Katsayılar W3C Audio EQ Cookbook'tan;
`alpha = sin(w0)/(2Q)`, `w0 = 2*pi*f0/fs`. Sabit katsayılı olduğu için modülasyonda kullanılmaz.

**Formant bankası:** 3–5 paralel BPF, `BW = 60..140 Hz` (F1–F3), `Q = f/BW`.
Zombi için tipik: F1 320–620 Hz, F2 900–1500 Hz, F3 2300–2900 Hz, F4 3300–3800 Hz.
Kadın/çocuk/erkek farkı tüm formantların `×1.15 / ×1.0 / ×0.88` ölçeklenmesiyle elde edilir.
Paralel yerine **cascade** (Klatt) kullanmak daha "konuşma" hissi verir; growl için paralel + her formantta
ayrı noise injection daha "gırtlak" hissi verir.

**Filter envelope:** cutoff'u **oktav uzayında** sürün: `fc(t) = fc0 * 2^(env_oct*e(t))`, `env_oct = 0..5`.
Lineer Hz modülasyonu düşük frekansta neredeyse hiçbir şey yapmaz — klasik hata.

### 2.5 Osilatörler, pitch envelope, FM/PM, waveshaping

- **Sine**: faz akümülatörü + `sin` (veya tablo + interpolasyon). **1024 nokta + lineer interp, 1 kHz'de ≈ −54 dB THD**
  verir; −80 dB hedefi için 4096 nokta + 4-nokta Hermite (kübik) interpolasyon gerekir. 48 kHz'de 2 kHz altı için
  4096 nokta + Hermite güvenli seçimdir.
- **Saw/Square**: **PolyBLEP** zorunlu, yoksa aliasing duyulur. Saw için
  `y = 2*phase-1 - polyblep(phase, dt)`, square için iki kenarda.
- **Wavetable**: mipmap'li (her oktav için ayrı band-limited tablo), aksi halde aliasing.
- **Pitch envelope**: `f(t) = f_end + (f_start - f_end)*exp(-t/tau)`, `tau = 3..15 ms`,
  `f_start/f_end = 1.5..4.0` (silah gövdesi), `8..20` (cartoon "pew"), `1.0` (nötr).
- **PM (FM değil, PM tercih edilir — drift yok):**
  `y = sin(2*pi*fc*t + I(t)*sin(2*pi*fm*t))`, `I = 0..10`.
  Yan bant genişliği ≈ `(I+1)*fm`; `I=5, fm=800 Hz` → 4.8 kHz bant. Aliasing'i önlemek için `fc + (I+1)*fm < 0.45*fs`.
- **Waveshaping**: `tanh(drive*x)/tanh(drive)`, `drive = 1..12`. Foldback: `x = fold(x*drive)`, eşik `0.7..1.5`.
  Bitcrush: `bits = 4..12`, `decim = 1..32` (sample & hold). **Hepsi 2–4× oversample edilmeden kullanılmamalı** (§8).

### 2.6 Granular synthesis

Zombi hırıltısı, kâğıt hışırtısı, enkaz, "sürünen kalabalık" için ana doku motoru.

| Parametre | Aralık | Varsayılan | Etki |
|---|---|---|---|
| `grain_ms` | 5–120 | 35 | kısa = pürüzlü/gırtlak, uzun = pürüzsüz/pad |
| `density` | 20–400 grain/s | 120 | `overlap = density*grain_ms/1000` |
| `spray` (position jitter) | 0–500 ms | 40 ms | "dağılmış" hissi |
| `pitch` | ±24 st | 0 | formant kayması da beraber |
| `pitch_jitter` | 0–2 st | 0.3 st | canlılık |
| `pan_spread` | 0–1 | 0.4 | genişlik |
| `window` | Hann/Tukey(0.5)/Gauss | Hann | kenar yumuşaklığı |
| `reverse_prob` | 0–0.3 | 0.05 | doku çeşitliliği |

**Overlap normalizasyonu:** toplam kazanç `1/sqrt(overlap)` ile ölçeklenir; aksi halde density değişince seviye patlar.
`overlap < 1` ise grain'ler arası boşluk duyulur (bazı efektlerde istenen "tıkırtı").

**Kaynak:** granular her zaman bir buffer'dan okur. Zombi için kaynak = sentetik glottal pulse train (aşağıda) veya
kısa bir "scream" kaydı. SoundGen'de her iki mod da olmalı: `source = synth | sample`.

### 2.7 Fiziksel modeller

**Karplus–Strong (telli/gerilmiş):**

```
y[n] = x[n] + g * 0.5*(y[n-N] + y[n-N-1])
f0 = fs / (N + 0.5 + d)      // d: allpass fractional delay (0..1)
```
`g = 0.995..0.9999`; `g`'yi `f0`'a bağlayın, yoksa tiz telde decay süresi değişir. Loop filter'ı bir kutuplu LP
`H(z) = (1-a)/(1-a*z^-1)` yapmak decay'i frekansa bağlı hale getirir (`a = 0.1..0.6`): tiz notalar daha hızlı söner.

**Waveguide (boru/hortum):** iki yönlü delay hattı + loss filter + junction. Boru için `f0 = c/(2L)`, `c = 343 m/s`,
`L = 0.2..3 m` → `f0 = 57..857 Hz`. Silah namlusu, susturucu, havalandırma borusu, "içi boş" sesler için.

**Stick–slip (gıcırtı / creak / kilit):**
Durum makinesi: yay kuvveti `F = k*(x - v*t)`; `F > mu_s*N` olduğunda kayma başlar, bağıl hız `v_th` altına
düşünce tekrar yapışır. Sonuç: `20..500 Hz` arası düzensiz darbe treni. Bu treni bir **modal bank** ile
filtrelemek (ahşap kasa modları) gerçek "gıcırtı"yı verir. Parametreler: `v_slip 0.5..8 mm/s`, `mu_s 0.2..0.8`,
`stiffness 200..5000 N/m`, `v_th 0.1..2 mm/s`. Yavaş `v_slip` = uzun gıcırtı, hızlı = "takırtı".

**Combustion / patlama (Friedlander):**

```
p(t) = Ps * (1 - t/td) * exp(-b*t/td),   0 <= t <= td
```
`Ps` = tepe basınç (normalize 1.0), `td = 1..10 ms` (pozitif faz), `b = 1..3` (sönüm şekli).
Bunu takip eden negatif faz (`td..3*td`, genlik `-0.3*Ps`) "N-wave" hissini verir. Düşük frekanslı gövde için
`40..90 Hz` sine + pitch drop, çatlaklar için Poisson darbe treni bir BPF'ten geçirilir.

**Türbülans / akış gürültüsü (pervane, rüzgâr):** kaynak = pink/brown noise; transfer fonksiyonu
`H(s) ≈ 1/(1 + s*tau_v)` ile vorteks sönümü; blade-pass rezonansı ayrı bir BPF ile eklenir.
Vorteks kopma frekansı `f_v = St*v/D` (Strouhal `St ≈ 0.2`, `v` = uç hızı, `D` = karakteristik çap).

### 2.8 Rotor: AM/FM, blade-pass, Doppler

```
BPF = (RPM/60) * blade_count          // örn. 2400 rpm, 2 kanat → 80 Hz
```
Harmonikler `n*BPF`, `n = 1..12`, kazanç `1/n^0.8`. AM derinliği `0.3..0.9`; tam AM (%100) "helikopter"
hisssini verir, %35 "uzak drone".

**Doppler (oyun içi loop için):**
`f' = f * c / (c - v_r)`, `c = 343 m/s`, `v_r` radyal hız. Loop'ta mutlak doppler yerine **pitch drift** kullanın:
`ratio(t) = 1 + 0.01*LFO(0.05..0.3 Hz) + random_walk(0.02..0.1 Hz)`. Bu, "motor yük değiştiriyor" hissi verir ve
loop noktasında phase sürekliliğini bozmaz.

**Loop phase sürekliliği:** loop uzunluğu **BPF periyodunun tam katı** olmalı; aksi halde her döngüde "vınlama"
duyulur. `loop_len = round(target_sec * BPF) / BPF`.

### 2.9 Reverb: convolution vs algoritmik

- **Kısa convolution (50–500 ms):** gövde rezonansı, silah kuyruğu, "iç mekân" hissi. 48 kHz'de 1024 tap = 21 ms;
  partitioned FFT (512 örnek) ile CPU maliyeti önemsiz. IR'ler SoundGen içinde **sentetik** üretilebilir
  (velvet noise + exp decay + spectral tilt) — dosya bağımlılığı yok.
- **Algoritmik (Freeverb / Schroeder):** gerçek zamanlı, modüle edilebilir, uzun kuyruk. Freeverb: 8 paralel
  lowpass-feedback comb + 4 seri allpass, her kanal için. 44.1 kHz sabitleri (48 kHz için `×1.0884` ölçeklenir):

| Comb (L) | 1116 | 1188 | 1277 | 1356 | 1422 | 1491 | 1557 | 1617 |
|---|---|---|---|---|---|---|---|---|
| Allpass (L) | 556 | 441 | 341 | 225 | | | | |

`stereospread = 23`, `feedback = 0.84` (room 0.5 karşılığı), `damping = 0.2..0.5`.
Combs'ların içinde **high-pass** olmalı, yoksa DC birikir ve kuyruk "şişer".

**Hibrit önerisi:** erken yansımalar (5–15 tap, 3–30 ms, ayrı gain/pan) + geç kuyruk (Freeverb veya velvet-noise
tabanlı). Silah sesinde erken yansıma olmadan kuyruk "yapıştırılmış" duyulur.

---

## 3. Kategori Reçeteleri

Notasyon: `KAYNAK -> FİLTRE -> ENV -> GAIN`. `[L]` = katman. Tüm gain'ler mix öncesi, dB.
Varsayılan master: `-1 dBTP`, kategori LUFS hedefi §6'da.

### 3.1 Tabanca (pistol, 9mm)

```
[L1 CLICK]   TransientGen(dur=1.2ms, curve=exp^4) -> Svf(BP, 4.2kHz, Q=0.9) -> ENV(0.05ms, 6ms) -> -8 dB
[L2 CRACK]   NoiseGen(white) -> Svf(BP, 3.5kHz, Q=0.7) + Svf(HP, 6kHz) -> ENV(0.1ms, 18ms) -> -6 dB
[L3 BODY]    Osc(sine, f0=110Hz) -> PitchEnv(start=3.0x, end=1.0x, tau=9ms) -> ENV(0.3ms, 55ms) -> -3 dB
[L4 THUMP]   NoiseGen(brown) -> Svf(LP, 180Hz, Q=0.9) -> ENV(0.5ms, 90ms) -> -10 dB
[L5 TAIL]    NoiseGen(pink) -> Svf(BP, 700Hz, Q=0.6) -> ENV(1ms, 320ms) -> Convolver(IR=0.25s outdoor) -> -14 dB
[L6 MECH]    ModalBank(f0=2.8kHz, ratios=bar, T60=45ms, Q=180) -> -22 dB   // sürgü/kovan mekanizması
SEND: Reverb(wet=0.12, room=0.35, damp=0.4)
```
Süre: 0.45–0.9 s. Gerçekçi modda L2'nin enerjisi 2–5 kHz'de merkezlenir; L5 kuyruğu 300 ms'yi geçmez (yakın mesafe).

### 3.2 Tüfek (rifle, 5.56 / 7.62)

```
[L1 CRACK]  NoiseGen(white) -> Svf(BP, 5.0kHz, Q=0.6) -> ENV(0.05ms, 10ms) -> 0 dB (en yüksek tepe)
[L2 SNAP]   TransientGen(0.6ms) -> Svf(HP, 9kHz) -> ENV(0.02ms, 3ms) -> -6 dB
[L3 BODY]   Osc(sine, 85Hz) + Osc(sine, 130Hz, -6dB) -> PitchEnv(2.6x→1.0x, tau=14ms) -> ENV(0.4ms, 70ms) -> -2 dB
[L4 BLAST]  NoiseGen(pink) -> Svf(LP, 1.2kHz, Q=0.5) -> ENV(0.8ms, 160ms) -> -8 dB
[L5 TAIL]   NoiseGen(pink) -> Svf(BP, 500Hz, Q=0.5) -> ENV(2ms, 650ms) -> Convolver(IR=0.6s valley) -> -12 dB
[L6 SHELL]  ModalBank(f0=2.2kHz, T60=120ms, Q=250) tetiklenme +85ms -> -26 dB
```
Tüfek vs tabanca ayrımı: **kuyruk uzunluğu** (650 vs 320 ms) ve **gövde pitch düşüşünün süresi** (14 vs 9 ms).
Tüfekte `L1` tepe noktası 3–6 kHz'de, tabancada 4–7 kHz'de olmalı.

### 3.3 Pompalı (shotgun)

```
[L1 BOOM]   NoiseGen(brown) -> Svf(LP, 400Hz, Q=0.7) -> ENV(1.5ms, 220ms) -> 0 dB
[L2 CRACK]  NoiseGen(white) -> Svf(BP, 2.4kHz, Q=0.5) -> ENV(0.1ms, 35ms) -> -4 dB
[L3 BODY]   Osc(sine, 62Hz) -> PitchEnv(1.8x→1.0x, tau=25ms) -> ENV(1ms, 140ms) -> -2 dB
[L4 DEBRIS] GrainCloud(source=noise, grain=25ms, density=90, spray=60ms) -> Svf(BP,1.8kHz,Q=1) -> -18 dB
[L5 TAIL]   Convolver(IR=0.9s outdoor) -> -10 dB
```
Pompalı "geniş" ve "alçak"tır; tüfek "keskin" ve "uzun". Aynı motor, farklı `fc`, `T60`, `pitch_env` ile üretilir.

### 3.4 Susturucu (suppressed)

```
[L1 PFFT]   NoiseGen(white) -> Svf(BP, 1.6kHz, Q=0.8) -> ENV(0.3ms, 40ms) -> -6 dB
[L2 THUMP]  Osc(sine, 95Hz) -> PitchEnv(1.4x→1.0x, 20ms) -> ENV(2ms, 90ms) -> 0 dB
[L3 MECH]   ModalBank(f0=1.9kHz, T60=60ms, Q=120) -> -12 dB
[L4 GAS]    NoiseGen(pink) -> Svf(BP, 900Hz, Q=1.4) -> ENV(3ms, 120ms) -> -14 dB
```
Kritik: 3–8 kHz enerjisi gerçekçi modda **−18..−25 dB** düşürülür. "Sinema susturucusu" (pfft) daha yüksek BPF
(1.6 kHz) ve daha uzun kuyruk ister; gerçek susturucu daha çok mekanik "tık" + gaz sesidir.

### 3.5 Şarjör / sürgü (reload, bolt)

```
[L1 LATCH]  TransientGen(0.8ms) -> Svf(BP, 3.2kHz, Q=1.2) -> ENV(0.05ms, 8ms) -> 0 dB
[L2 METAL]  ModalBank(f0=2.4kHz, ratios=bar, T60=70ms, Q=300, mistune=1.5%) -> -6 dB
[L3 SLIDE]  NoiseGen(white) -> Svf(BP, 2.2kHz, Q=2.5) -> ENV(4ms, 45ms) + amplitude wobble(18Hz, 30%) -> -12 dB
[L4 SPRING] ModalBank(f0=880Hz, T60=180ms, Q=90) -> -18 dB
Zamanlama: LATCH @0ms, SLIDE @35–70ms, ikinci LATCH @110–160ms
```
İki "tık" arası süre `60..160 ms` rastgeleleştirilir; aynı ses tekrarında bu, "kopya" hissini yok eder.

### 3.6 Kovan düşmesi (casing drop)

```
[L1 TINK]   TransientGen(0.5ms) -> Svf(HP, 5kHz) -> ENV(0.02ms, 4ms) -> -8 dB
[L2 RING]   ModalBank(f0=2600Hz, ratios=1, 2.76, 5.40, 8.93, T60=180ms, Q=600, mistune=2%) -> 0 dB
[L3 BOUNCE] aynı bank, 4–6 tekrar: t_k = t_0 + sum(isi), isi *= 0.62..0.75, gain *= 0.55..0.7
```
Kovanın metal oranları **bar** serisidir; `T60 = 120..400 ms`. Zemin tipi (beton/ahşap/çamur) `L2`'nin
`fc`'sini ve `T60`'ını değiştirir, `L1`'i değil.

### 3.7 Zombi hırıltısı (growl)

```
[L1 GLOTTAL] OscPulse(f0=72Hz, jitter=3%, shimmer=6%) -> FormantBank(F1=480,F2=1150,F3=2500,BW=90) -> -3 dB
[L2 SUB]     Osc(sine, 36Hz) + Osc(sine, 54Hz, -8dB) -> ENV(80ms, 900ms) -> -8 dB
[L3 CREAK]   StickSlip(v_slip=2mm/s, stiffness=900) -> ModalBank(ahşap-benzeri, f0=420Hz) -> -16 dB
[L4 RASP]    GrainCloud(source=L1, grain=28ms, density=140, spray=35ms, pitch_jitter=0.4st) -> -10 dB
[L5 BREATH]  NoiseGen(pink) -> Svf(BP, 1.4kHz, Q=1.2) -> ENV(60ms, 700ms) -> -14 dB
Süre: 0.9–2.4 s; f0 konturu: 72→58→66 Hz (yorgun, düşen)
```
"İnsan ama değil" hissi: formantlar insan aralığında **ama F1 düşük**, `f0` düşük, ve `L3` creak katmanı.
`shimmer` (genlik jitter) %5–10 arası; %15 üstü "robot" olur.

### 3.8 Zombi çığlığı (scream)

```
[L1 VOICE]  OscSaw(f0=420Hz) + PM(fm=420, index=1.5) -> FormantBank(F1=700,F2=1700,F3=2900) -> -2 dB
[L2 PITCH]  PitchEnv(420→620→380Hz, 0.9s, vibrato 5.5Hz/±18 cent) -> -0 dB
[L3 DIST]   Shaper(tanh, drive=3.2) -> Svf(LP, 6.5kHz, Q=0.7) -> -8 dB
[L4 TEAR]   GrainCloud(source=L1, grain=12ms, density=220, spray=18ms) -> -12 dB
[L5 AIR]    NoiseGen(white) -> Svf(BP, 3.2kHz, Q=1.5) -> ENV(40ms, 500ms) -> -20 dB
```
Süre 0.7–1.9 s. Çığlıkta `L3` (distortion) **şart**: temiz formant sentezi "opera" gibi duyulur, "zombi" gibi değil.

### 3.9 Zombi ısırma (bite / feed)

```
[L1 SNAP]   TransientGen(1.5ms) -> Svf(BP, 1.1kHz, Q=1.4) -> 0 dB
[L2 WET]    NoiseGen(pink) -> Svf(BP, 600Hz, Q=3) sweep → 1800Hz over 60ms -> ENV(3ms, 90ms) -> -4 dB
[L3 SQUELCH] Osc(sine, 180Hz) -> PitchEnv(1.6x→1.0x, 40ms) + AM(28Hz, 40%) -> -10 dB
[L4 TEAR]   GrainCloud(grain=8ms, density=260, spray=25ms) -> Svf(BP, 2.4kHz, Q=0.8) -> -14 dB
```

### 3.10 Zombi ölümü (death)

```
[L1 GROAN]  growl zinciri, f0 72→42Hz, süre 1.6s, formantlar %15 düşük -> -2 dB
[L2 FALL]   NoiseGen(brown) -> Svf(LP, 120Hz) -> ENV(20ms, 400ms) -> -6 dB
[L3 GURGLE] NoiseGen(pink) -> Svf(BP, 400Hz, Q=4) + AM(9Hz, 60%) -> ENV(200ms, 1.2s) -> -12 dB
[L4 LAST]   breath katmanı, 0.4s, giderek kısılan -> -18 dB
```

### 3.11 Et isabeti (flesh hit)

```
[L1 SLAP]   TransientGen(2ms) -> Svf(BP, 900Hz, Q=0.8) -> -2 dB
[L2 THUD]   Osc(sine, 95Hz) -> PitchEnv(2.2x→1.0x, 18ms) -> ENV(1ms, 80ms) -> 0 dB
[L3 SQUISH] NoiseGen(pink) -> Svf(BP, 1.6kHz, Q=1.8) sweep ↓ 800Hz, 70ms -> -8 dB
[L4 WET]    GrainCloud(grain=6ms, density=180, spray=20ms) -> Svf(LP, 3kHz) -> -16 dB
```
Et = **düşük Q** (2–6) + yüksek nem oranı. Kuru et (kemikli) için `L1` güçlenir, `L3` zayıflar.

### 3.12 Kemik kırılması (bone break)

```
[L1 CRACK]  TransientGen(0.4ms) -> Svf(HP, 3kHz) -> 0 dB
[L2 SPLINTER] ModalBank(f0=3.1kHz, ratios=plate, T60=35ms, Q=800, mistune=4%) -> -4 dB
[L3 GRIND]  NoiseGen(white) -> Svf(BP, 4.5kHz, Q=1.2) -> ENV(2ms, 45ms) -> -8 dB
[L4 DEBRIS] 3–6 ayrı transient, 20–90ms aralıklarla, gain -6..-18dB
```

### 3.13 Kafa patlaması (head pop)

```
[L1 BURST]  Blast(Ps=1, td=3ms, b=2.2) -> Svf(LP, 2.5kHz) -> 0 dB
[L2 WET]    NoiseGen(pink) -> Svf(BP, 700Hz, Q=1.1) -> ENV(5ms, 260ms) -> -4 dB
[L3 SPLAT]  GrainCloud(grain=15ms, density=160, spray=80ms) -> Svf(BP,1.2kHz,Q=0.7) -> -8 dB
[L4 BONE]   ModalBank(f0=2.4kHz, T60=60ms, Q=500) -> -14 dB
[L5 TAIL]   Convolver(IR=0.4s, small room) -> -12 dB
```
Bu kategoride **gerçekçilik tavanı** düşüktür; oyunlarda genelde stilize edilir (`style 0.5–0.75`).

### 3.14 Adımlar

| Zemin | Katmanlar |
|---|---|
| **Beton** | `Transient(0.8ms)->BPF(3.5k,Q=1.2)` + `Noise(white)->HP(1.5k)->ENV(1ms,45ms)` + `ModalBank(180Hz, T60=60ms, Q=25)` @ −10 dB |
| **Çimen** | `Noise(pink)->BPF(2.2k,Q=0.7)->ENV(6ms,110ms)` + `GrainCloud(grain=20ms, density=70)` @ −6 dB + hafif `ModalBank` yok |
| **Çamur** | `Noise(brown)->LP(700Hz)->ENV(10ms,160ms)` + `BPF(400Hz,Q=3)` sweep ↓ + `AM(12Hz,25%)` "squelch" |
| **Metal ızgara** | `ModalBank(f0=900Hz, ratios=plate, T60=250ms, Q=400)` + transient |

Adım süresi 40–120 ms; çift adım arası 380–650 ms (yürüme), 240–380 ms (koşma).

### 3.15 Ceset düşmesi (body fall)

```
[L1 THUD]   Osc(sine, 55Hz) -> PitchEnv(1.5x→1.0x, 30ms) -> ENV(3ms, 200ms) -> 0 dB
[L2 IMPACT] NoiseGen(brown) -> Svf(LP, 300Hz) -> ENV(2ms, 120ms) -> -6 dB
[L3 CLOTH]  NoiseGen(pink) -> Svf(BP, 1.8kHz, Q=0.6) -> ENV(15ms, 250ms) -> -14 dB
[L4 SETTLE] 2–3 küçük transient (60–200ms sonra), gain -12..-24dB
```

### 3.16 Ahşap kasa: gıcırtı / açılma / parçalanma

**Gıcırtı (creak):**
```
StickSlip(v_slip=1.2mm/s, stiffness=1400, mu_s=0.45) -> ModalBank(ahşap: 1,2.32,3.71,5.14,6.90; f0=320Hz; T60=0.5s; Q=45)
  -> Svf(BP, 900Hz, Q=0.8) -> ENV(fade-in 120ms, out 300ms) -> -6 dB
```
Gıcırtının hızı `v_slip` ile; tonu `f0` ve `stiffness` ile. Süre 0.4–2.5 s.

**Açılma (open):**
```
[L1 LATCH]  Transient(0.6ms) -> BPF(2.6k, Q=1.5) -> -4 dB
[L2 CREAK]  yukarıdaki gıcırtı, 250–700ms -> -8 dB
[L3 WHOOSH] Noise(pink) -> BPF(700Hz, Q=1.1) sweep ↑ 1.8kHz over 180ms -> -14 dB
[L4 WOOD]   ModalBank(f0=240Hz, T60=300ms, Q=30) tetik @ kapanış -> -10 dB
```

**Parçalanma (shatter):**
```
[L1 CRACK]  Blast(td=1.5ms) -> HP(2kHz) -> 0 dB
[L2 SPLINTER] 12–25 ayrı ModalBank grain'i (f0 400–2500Hz, T60 20–90ms) Poisson dağılımıyla 0–600ms -> -8 dB
[L3 DEBRIS] GrainCloud(white, grain=6ms, density=300, spray=120ms) -> BPF(3kHz) -> -12 dB
[L4 TAIL]   Convolver(IR=0.35s wood room) -> -14 dB
```

### 3.17 Kilit (lock click)

```
[L1 CLICK]  Transient(0.3ms) -> Svf(BP, 4.5kHz, Q=2) -> 0 dB
[L2 TUMBLER] ModalBank(f0=3.4kHz, ratios=bar, T60=25ms, Q=900) -> -8 dB
[L3 BODY]   ModalBank(f0=420Hz, T60=90ms, Q=40) -> -12 dB
```
Toplam süre 25–60 ms. İki kademeli kilitte ikinci click 45–90 ms sonra, `-3 dB`.

### 3.18 Konfeti / parti bombası (confetti popper)

```
[L1 POP]    Blast(Ps=1, td=1.2ms, b=3.0) -> Svf(HP, 1.5kHz) -> 0 dB
[L2 AIR]    NoiseGen(white) -> Svf(BP, 2.8kHz, Q=0.6) -> ENV(0.2ms, 30ms) -> -4 dB
[L3 FLUTTER] GrainCloud(source=paper_noise, grain=18ms, density 40→900 grain/s over 250ms, spray=60ms)
            -> Svf(BP, 3.5kHz, Q=0.9) -> ENV(5ms, 700ms) -> -8 dB
[L4 RUSTLE] NoiseGen(velvet, 1800 pps) -> Svf(HP, 2kHz) -> ENV(20ms, 1.2s) -> -16 dB
[L5 SPARKLE] 3–7 kısa sine "ping" (2.5–6 kHz), rastgele 150–900ms, gain -22..-30dB
```
Kritik: `L3`'ün **density rampası** (40→900) "kâğıtların açılması" hissini verir; sabit density "shaker" gibi duyulur.

### 3.19 Pervane döngüsü (propeller loop)

```
[L1 BPF TONE] NoiseGen(pink) -> Svf(BP, BPF, Q=6) -> AM(f=BPF, depth=0.75) -> -3 dB
[L2 HARMONICS] 5 sine @ n*BPF (n=1..5), gain 1/n^0.9, her biri ±0.3% detune -> -6 dB
[L3 THRUM]  Osc(sine, BPF/2) -> AM(BPF, 0.5) -> -12 dB
[L4 AIR]    NoiseGen(white) -> Svf(HP, 4kHz) -> -24 dB
[L5 DRIFT]  tüm L1–L4: pitch_ratio = 1 + 0.01*LFO(0.12Hz) + walk
```
`BPF = RPM/60*blades`. Örnek: 2400 rpm, 2 kanat → 80 Hz; 4 kanat → 160 Hz. Loop uzunluğu `n/BPF` tam katı.
Stereo: iki kanal arasında `8–25°` faz farkı + hafif farklı `drift` → genişlik.

### 3.20 Drone motor

```
[L1 WHINE]  Osc(saw, f0=BPF*2) -> Svf(BP, 2.8kHz, Q=8) -> -10 dB
[L2 BUZZ]   Osc(saw, f0=BPF) + PM(ratio=1.01, index=0.8) -> -8 dB
[L3 BODY]   NoiseGen(brown) -> Svf(LP, 400Hz) -> -6 dB
[L4 WOBBLE] AM(3.5Hz, 18%) + pitch drift ±0.6%
```
Drone'da `BPF` 130–600 Hz (4000–9000 rpm, 2–4 kanat); "quadcopter" için `BPF` 120–200 Hz ve `L1` baskın.

### 3.21 Whoosh

```
[L1 CORE]   NoiseGen(pink) -> Svf(BP, fc: 250→3800Hz over 420ms, Q=1.8) -> ENV(120ms in, 300ms out)
[L2 SUB]    NoiseGen(brown) -> LP(200Hz) -> ENV(200ms in, 250ms out) -> -8 dB
[L3 AIR]    NoiseGen(white) -> HP(6kHz) -> ENV(50ms in, 150ms out) -> -16 dB
[PAN]       L/R'ye 60–120ms gecikmeli kopya (Haas) + ters yön pan eğrisi
```
Süre 250–900 ms. `fc` eğrisi lineer değil **üstel** olmalı (`fc = fc0 * (fc1/fc0)^t`), yoksa "yanlış" duyulur.

### 3.22 UI click / coin / level-up

**UI click** (30–60 ms):
```
Transient(0.4ms) -> Svf(BP, 2.2kHz, Q=1.6) -> ENV(0.05ms, 12ms) + ModalBank(f0=1.8kHz,T60=25ms,Q=200)@-10dB
```

**Coin** (350–500 ms, iki nota):
```
Osc(square, 987.77Hz = B5) -> ENV(2ms, 70ms) -> -3 dB
Osc(square, 1318.51Hz = E6) @ +75ms -> ENV(2ms, 300ms) -> -3 dB
+ HP(400Hz), hafif Shaper(drive=1.3)
```
Bu aralık (B5→E6, tam dörtlü) endüstri standardı "ödül" algısıdır; alternatif: C6→G6.

**Level-up** (1.2–2.0 s): 6 notalı arpej (C-E-G-B-D-G), her nota 70–90 ms arayla, her notada `Osc(saw)+PM(index 0.6)`,
üstüne 2–3 oktav yukarıda `sine` shimmer, son nota 400 ms kuyruk + reverb %25.

### 3.23 Ortam katmanları

**Rüzgâr (20–60 s loop):**
```
[L1 BED]  NoiseGen(brown) -> Svf(LP, fc=350..1100Hz, LFO 0.07Hz) -> -6 dB
[L2 GUST] NoiseGen(pink) -> Svf(BP, 700Hz, Q=0.8) -> AM(0.13Hz, 55%) -> -10 dB
[L3 WHISTLE] NoiseGen(white) -> Svf(BP, 2.4kHz, Q=14) -> AM(0.09Hz, 40%) -> -26 dB
[L4 DEBRIS] Poisson grain'ler (rüzgârla savrulan yaprak/taş), 0.5–2/s -> -30 dB
```
Loop dikişi: 2–4 s crossfade, equal-power; `L1`'in LFO periyodu loop uzunluğunu tam böler.

**Uzak savaş (20–40 s):**
```
Poisson olayları (2–6 s aralıklı): gunshot zinciri -> Svf(LP, 900Hz) -> Convolver(IR=1.8s) -> -24 dB
+ sürekli alçak uğultu (Noise brown, LP 90Hz, -34 dB) + ara sıra patlama (Blast, LP 300Hz, -20 dB)
```

---

## 4. Gerçekçilik ↔ Stil Morph Ekseni

### 4.1 Tanım

`style ∈ [0,1]`, `0 = fiziksel/gerçekçi`, `1 = tam cartoon`. Ancak **tek bir global lineer interpolasyon yanlıştır**:
gerçekçilikten uzaklaşırken bazı parametreler *artmalı*, bazıları *azalmalı*, bazıları *yorunge değiştirmeli*.

Model:

```
p(style) = p0 + (p1 - p0) * curve(style)
curve: linear | expo(k) | smoothstep | stepped(n)
```

Her sürücü için `curve` ayrı seçilir. Örnek: `pitch_env_depth` lineer artar; `modal_Q` üstel düşer (duyulabilir
fark ilk %30'da); `reverb_wet` smoothstep ile son %40'ta patlar.

### 4.2 Sürücü tablosu (tüm kategorilerde geçerli 12 çekirdek sürücü)

| # | Sürücü | style=0 | style=1 | curve | Neden |
|---|---|---|---|---|---|
| D1 | `pitch_env_depth` | ×1.0 | ×2.2–3.5 | expo(2) | çizgi film "pew/boing" |
| D2 | `modal_Q` | fiziksel (20–1500) | ×0.25 | expo(1.5) | daha "tık", daha az "çın" |
| D3 | `modal_mistune` | %0.5–2 | %4–7 | linear | toy/ucuz nesne |
| D4 | `noise_brightness` (spectral tilt) | −1 dB/oct | +2 dB/oct | linear | parlak, agresif |
| D5 | `transient_ms` | 0.3–2 | 2–6 | linear | yumuşak, "squash" |
| D6 | `saturation_drive` | 1.0 | 2.5–5 | expo(2) | kompres/yapışkan |
| D7 | `bit_depth` / `decim` | 24 bit / 1 | 8–10 bit / 2–4 | stepped(4) | retro grit |
| D8 | `stereo_width` | 1.0 | 1.6–1.9 | smoothstep | abartılı genişlik |
| D9 | `reverb_wet` | kategoriye göre | ×2.0–2.8 | smoothstep | "büyük sahne" |
| D10 | `duration_scale` | 1.0 | 1.15–1.6 | linear | uzayan kuyruk |
| D11 | `layer_count` | tam | −30% (az katman) | stepped(3) | basitleşmiş, okunur |
| D12 | `tonal_layer_gain` | −∞ (yok) | −6..−12 dB | smoothstep | müzikal/pitched bileşen |

### 4.3 Kategori bazlı eşleme (somut değerler)

| Kategori | style=0 | style=1 | Ek not |
|---|---|---|---|
| **Tabanca** | body 110Hz, pitch 3.0x→1.0x/9ms; crack 3.5kHz Q0.7; tail 320ms; wet .12 | body 180Hz, pitch 4.5x→1.0x/25ms; crack 1.8kHz Q3 (lazer); tail 520ms; wet .30; drive 3.5; 10-bit | "pew" = dar bant + derin pitch drop |
| **Tüfek** | 85/130Hz, tau 14ms, tail 650ms | 150Hz, tau 35ms, tail 900ms, + sine "zap" @−10dB | tüfek cartoon'da "lazer tüfeği" |
| **Pompalı** | 62Hz, td 220ms | 95Hz, pitch 2.5x, tail 1.4s, + "boom" AM | |
| **Susturucu** | 3–8kHz −20dB, tail 120ms | "pfft" BPF 2.4kHz Q2.5, tail 400ms, + tongue click | |
| **Zombi growl** | f0 72Hz, jitter 3%, formant insan | f0 110Hz, jitter 12%, formant ×1.2, + ring-mod 40Hz | ring-mod = "alien" |
| **Zombi scream** | drive 3.2, vibrato 5.5Hz | drive 6, vibrato 9Hz/±40 cent, + PM index 4 | |
| **Et isabeti** | Q 3–6, squish 70ms | Q 12, squish 200ms, + "slap" 400Hz sine | |
| **Kemik** | mistune 4%, T60 35ms | mistune 8%, T60 90ms, + "xylophone" tonal | |
| **Kafa patlaması** | 5 katman, wet .2 | 3 katman, wet .5, + "splat" pitch drop | |
| **Adım (beton)** | 3 katman, 45ms | tek "tık", 90ms, + tonal 300Hz | |
| **Kasa gıcırtısı** | stick-slip + ahşap modları | "keman" tonu: KS string 220Hz + vibrato | |
| **Konfeti** | grain 18ms, density 40→900 | grain 40ms, density 30→300, + "boing" 200Hz | |
| **Pervane** | BPF Q6, drift %1 | BPF Q2, drift %6, + "wah-wah" LFO 3Hz | |
| **Whoosh** | 250→3800Hz, 420ms | 150→6000Hz, 700ms, + tonal sweep sine | |
| **UI click** | 12ms, 2.2kHz | 30ms, 1.2kHz, + "boop" pitch drop | |
| **Coin** | B5→E6 square | + 3. oktav shimmer, 16-bit crush, tail 600ms | |
| **Rüzgâr** | pink+brown, LP 350–1100 | + "howl" BPF Q20 700Hz, LFO 0.4Hz | |

**Global stil vektörü** (kategori tablosunun üstüne biner): `style_global = clamp(style_category + bias, 0, 1)`.
UI'da tek slider; altında 12 sürücü çubuğu (advanced panel).

### 4.4 Neden bu eksen çalışır (psikoakustik gerekçe)

- Gerçekçilik **spektral yoğunluk + modal oranlar + mikro-düzensizlik** üçlüsünden gelir. Stil, bu üçünü
  *bozarak* değil, **baskınlaştırarak** elde edilir: tonal bileşenler öne çıkar, gürültü azalır, pitch eğrileri abartılır.
- Kulak, 1–4 kHz arasındaki enerjiyi "saldırganlık", 60–120 Hz'i "ağırlık", 200–800 ms kuyruğu "mekân" olarak okur.
  Stil ekseni bu üç algısal boyutu **bağımsız** sürer (D4, D1, D9).
- `bitcrush` ve `saturation` tek başına "retro" verir ama "cartoon" vermez; cartoon için **pitch envelope derinliği**
  ve **modal Q düşüşü** (tıkırtı) gerekir. Bu yüzden D1 ve D2 tabloda ilk sırada.

---

## 5. Varyasyon ve Rastgeleleştirme

### 5.1 Seed mimarisi

```
master_seed (uint64)
  └─ variation_seed = splitmix64(master_seed ^ variation_index)
       └─ layer_seed = splitmix64(variation_seed ^ hash64(layer_id))
            └─ param_seed = splitmix64(layer_seed ^ hash64(param_id))
```
- Aynı `variation_index` + aynı parametre vektörü → **bit-identical** render (float determinism için sabit işlem sırası,
  `-ffast-math` yok, FMA varsa sabit).
- Katman eklemek diğer katmanların gürültüsünü değiştirmez → kullanıcı bir katmanı silince diğerleri "kaymaz".

### 5.2 Hangi parametreler rastgeleleştirilir

| Parametre | Dağılım | Aralık | Gerekçe |
|---|---|---|---|
| Frekans (modal `f0`, `fc`, `BPF`) | log-uniform | ±%8 | algısal olarak eşit aralık |
| Mod oranları | normal(1, 0.012) | ±%3 | mistuning |
| Gain'ler | normal(0, 1.2 dB) | ±3 dB | seviye hafızası |
| Süreler (`T60`, `decay`) | log-normal(σ=0.12) | ×0.8–1.25 | |
| Zamanlama (olay aralıkları) | uniform | ±%15 | "insan" hissi |
| `density`, `spray` | uniform | ±%25 | doku çeşitliliği |
| `pitch` (granular) | normal(0, 0.25 st) | ±1 st | |
| `pan` | uniform | ±0.35 | |
| `jitter`, `shimmer` | uniform | ±%30 (göreli) | |

**Kesinlikle rastgeleleştirilmeyecekler:** `style`, kategori kimliği, loudness hedefi, kanal sayısı, ana `f0` merkezi
(bir "aile" aynı enstrümanı temsil etmeli).

### 5.3 "Family" üretimi (kardeş varyasyonlar)

Amaç: 12 ses, aynı nesnenin 12 farklı vuruşu gibi duyulsun; 12 farklı nesne gibi değil.

1. **Makro uzay**: 4 eksen — `size` (0..1), `brightness`, `mess` (düzensizlik), `age` (yaş/yıpranma).
2. **Örnekleme**: Latin Hypercube veya Sobol, 12 nokta. Rastgele uniform'dan daha iyi yayılım.
3. **Kısıt**: her varyasyonun spektral centroid'i aile ortalamasının **±%15**'i içinde; süre **±%20**.
   Bu kısıtı sağlamayan aday yeniden örneklenir (rejection sampling, max 20 deneme).
4. **Algısal güvenlik**: frekans ±%5, seviye ±0.7 dB, süre ±%8 (Weber fraksiyonları) — bunun altındaki farklar
   "aynı ses" sayılır ve aile içinde gereksiz tekrar oluşturur.
5. **Çıktı**: her varyasyon için `seed`, üretim parametreleri JSON'u ve LUFS ölçümü saklanır.

### 5.4 Tekrar önleme (oyun içi)

Oyun tarafında 3–5 varyasyon döngüsel çalınır; SoundGen her varyasyona **pitch** (`±40 cent`) ve **gain** (`±1.5 dB`)
randomizasyonu önerebilir (meta dosyasında `playback_hints`). Ayrıca "round-robin + no-repeat-2" kuralı.

---

## 6. Loudness ve Dinamikler

### 6.1 Ölçüm zinciri (ITU-R BS.1770 / EBU R128)

- **K-weighting**: 1. aşama yüksek-shelf (+4 dB, `fc ≈ 1681 Hz`, Q 0.707), 2. aşama high-pass (`fc ≈ 38 Hz`, Q 0.5).
- **Gating**: 400 ms bloklar, %75 overlap; absolute gate `−70 LUFS`; relative gate ungated ortalama `−10 LU` altı.
- **Integrated (LUFS-I)** en az 400 ms gerektirir; kısa SFX için **LUFS-M max (400 ms)** veya **LUFS-S (3 s)** kullanılır.
  **Kritik ayrım:** 3 s'lik short-term penceresi, 0.5 s'lik bir silah sesinde %80 sessizlik içerir ve ölçümü
  olduğundan ~8 dB düşük gösterir. Bu yüzden SoundGen'de varsayılan metrik:
  **`active_lufs` = sesin aktif bölgesi (ilk ve son −50 dBFS geçişi arası) üzerinden LUFS-M max**;
  > 1 s seslerde `short_term_max`, loop'larda tüm döngü üzerinden `integrated`.
- **True peak**: 4× oversample (polyphase FIR) ile ölçülür; sample peak yeterli değildir (inter-sample peak 1–2 dB yukarı çıkabilir).

### 6.2 Hedefler

> Not: Oyun SFX için tek bir zorunlu standart yoktur; EBU R128'in `−23 LUFS-I` değeri **broadcast** içindir.
> Aşağıdaki aralıklar yaygın oyun prodüksiyon pratiğidir ve **kategori-relative** okunmalıdır (hepsi aynı projede
> birbirine göre anlamlıdır). SoundGen bunları "preset hedef" olarak sunmalı, kullanıcı değiştirebilmelidir.

| Kategori | Hedef (`active_lufs` = LUFS-M max) | True peak | Not |
|---|---|---|---|
| Silah (tabanca/tüfek) | −14 … −17 | ≤ −1 dBTP | hero SFX, en yüksek |
| Pompalı / patlama | −13 … −16 | ≤ −1 dBTP | |
| Susturucu | −20 … −24 | ≤ −1 dBTP | |
| Zombi çığlık | −15 … −18 | ≤ −1 dBTP | |
| Zombi growl | −18 … −21 | ≤ −1 dBTP | |
| Et / kemik / kafa | −14 … −18 | ≤ −1 dBTP | |
| Adım | −20 … −24 | ≤ −1 dBTP | |
| Ceset düşmesi | −18 … −21 | ≤ −1 dBTP | |
| Kasa / kilit | −18 … −22 | ≤ −1 dBTP | |
| Konfeti | −16 … −19 | ≤ −1 dBTP | |
| Pervane / drone (loop) | −20 … −24 | ≤ −1 dBTP | loop loudness sabit olmalı |
| Whoosh | −18 … −22 | ≤ −1 dBTP | |
| UI click | −20 … −24 | ≤ −1 dBTP | |
| Coin / level-up | −18 … −21 | ≤ −1 dBTP | |
| Ortam bed (loop) | −26 … −32 | ≤ −3 dBTP | arka plan, dinamik pay bırakır |

**Tutarlılık kuralı:** aynı kategorideki tüm varyasyonlar arasında fark **≤ 1.0 LU** olmalı; yoksa oyunda
"bazı ateşler patlamıyor" hissi oluşur.

### 6.3 İşleme zinciri (export)

```
1. DC blocker:     2. derece Butterworth HP, fc = 20 Hz (sub-bass korunur) veya 30 Hz (gürültülü sahneler)
2. Resonant peak:  opsiyonel notch (odalama frekansı), Q 8–12, −3..−6 dB
3. Fade:           in 1–3 ms raised-cosine; out 5–30 ms (loop ise crossfade 20–50 ms, equal-power)
4. Loudness:       `active_lufs` hedefe ölçekle (gain, dB)
5. True-peak limiter: lookahead 2 ms, release 30–80 ms, ceiling −1.0 dBTP
6. Dither:         sadece 16-bit'e düşerken, TPDF ±1 LSB; 24-bit'te dither YOK
7. Write:          48 kHz, 24-bit PCM WAV (oyun), mono veya stereo
```

**Sıra önemli:** limiter'dan sonra dither; loudness'tan önce fade. Fade'i en sona koymak, fade-in'in tepe değerini
bozar ve "click" riskini artırır.

**Mono uyumluluğu:** stereo efektlerde `mid = (L+R)/2` üzerinde de loudness kontrol edilmeli; faz ters çevrilmiş
katmanlar mono'da yok olur. `correlation meter` eklensin (hedef: −0.2 … +1.0; sürekli negatif = mono'da iptal).

---

## 7. Performans Bütçesi

### 7.1 Gerçek zamanlı kısıtlar (masaüstü, Windows)

| Block | Süre @48 kHz | Kullanılabilir CPU (tek çekirdek, %20) |
|---|---|---|
| 64 | 1.33 ms | 0.27 ms |
| **128** | **2.67 ms** | **0.53 ms** |
| 256 | 5.33 ms | 1.07 ms |
| 512 | 10.67 ms | 2.13 ms |

Öneri: **256 örnek** (`5.33 ms`) — 128'e göre 2× daha az callback overhead, 512'ye göre daha düşük latency.
Audition için toplam gecikme `block + ASIO buffer + UI` ≈ 10–15 ms; bu, "anında çalıyor" hissi için yeterlidir.

### 7.2 Voice bütçesi (tek çekirdek, 3.0 GHz, AVX2)

| Bileşen | Adet | Örnek başına işlem | Toplam |
|---|---|---|---|
| Modal bank (32 mod) | 16 voice | 32×5 FLOP | 2 560 FLOP/örnek |
| Granular (64 grain) | 4 cloud | 64×(2 okuma + 4 FLOP) | ~1 536 |
| SVF | 48 | 9 FLOP | 432 |
| Osc | 32 | 6 FLOP | 192 |
| Reverb (Freeverb) | 1 | ~40 FLOP | 40 |
| Convolution (512 FFT) | 2 | amortize ~25 FLOP | 50 |

Toplam ≈ **4 800 FLOP/örnek** → 48 kHz'de **≈ 230 MFLOP/s**. 3 GHz tek çekirdekte AVX2 ile teorik 40+ GFLOP/s;
gerçekçi olarak **%4–10 CPU**. 32 voice'a kadar rahat.

**Sınırlar:** `MAX_VOICES = 64`, `MAX_MODES_PER_VOICE = 64`, `MAX_GRAINS_PER_CLOUD = 128`, `MAX_CLOUDS = 8`.
Voice stealing: en düşük öncelikli + en eski; 5 ms fade-out ile (click yok).

### 7.3 SIMD-friendly tasarım

- **SoA (structure of arrays)**: `float a1[MAX_MODES]`, `float a2[...]`, `float y1[...]`, `float y2[...]`.
  8 modu tek AVX register'da işlemek için **mükemmel**; AoS (struct) 3–4× yavaş.
- **Branch-free**: mod başına `if (active)` yok; pasif modlar `g=0` ile çarpılır veya `active_mask` ile maskelenir.
- **Katsayı önceden hesaplanır** (`prepare`), ses thread'inde `cos/exp` yok. Modülasyon gerekiyorsa
  `tan` yerine yaklaşım (`tan(x) ≈ x + x³/3 + 2x⁵/15`, `|x| < 0.8`) veya tablo + interpolasyon.
- **FFT**: 512/1024 nokta, radix-2, gerçek-girişli (rfft); partitioned convolution için **uniform partition** yeterli.
- **Denormal koruması**: `_MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON)` + `_MM_SET_DENORMALS_ZERO_MODE(_MM_DENORMALS_ZERO_ON)`
  (Windows/MSVC: `_controlfp(_DN_FLUSH, _MCW_DN)`). Alternatif: her feedback yolunda `+1e-20f` DC offset (daha yavaş, ama taşınabilir).

### 7.4 Thread modeli

| Thread | Sorumluluk | Yasak |
|---|---|---|
| **Audio** | `process()` çağrıları, parametre yumuşatma, voice yönetimi | allocation, lock, dosya I/O, `printf`, `std::function` |
| **Worker (render)** | offline render, FFT/IR hazırlığı, LUFS analizi, waveform/spectrogram, preset kaydetme | audio thread'i bloklamak |
| **UI** | 30–60 fps görselleştirme, etkileşim | DSP state'e doğrudan yazmak |

İletişim: **SPSC lock-free ring buffer** (1024 event). Audio thread sadece okur; worker/UI sadece yazar.
Parametre güncellemeleri `atomic<float>` + `param_version` ile; audio thread block başında snapshot alır.

### 7.5 Offline render stratejisi

Audition her zaman **offline** render edilir (block 1024, gerçek zamandan bağımsız), sonucu ring buffer'a yazıp
çalar. Böylece:
- Ağır ama kaliteli algoritmalar (8× oversampling, uzun convolution) gerçek zamanlı bütçeye takılmaz.
- Aynı kod yolu export'ta da kullanılır → "duyduğun = aldığın".
- Kullanıcı slider'ı çekerken 20–40 ms'de yeniden render (debounce) yeterli.

---

## 8. Uygulama Tuzakları

| Tuzak | Belirti | Çözüm |
|---|---|---|
| **Aliasing (waveshaper)** | `tanh`/foldback'te tiz "cızırtı", özellikle yüksek drive'da | 2–4× oversample + halfband FIR; veya ADAA (antiderivative anti-aliasing). `bitcrush`'ta decimation zaten aliasing üretir — bilinçli, ama `drive` ile birlikte kullanma |
| **Aliasing (osc)** | saw/square'da "metalik" yanlış tonlar | PolyBLEP / mipmap wavetable; `f0 > fs/6`'da BLEP şart |
| **Filter kararsızlığı** | yüksek Q + hızlı cutoff modülasyonunda patlama, "whoop" | TPT/zero-delay SVF; `fc ≤ 0.45*fs` clamp; `Q ≤ 40`; modülasyon hızı ≤ 1 oktav/ms; filtre içinde soft-clip (`tanh`) |
| **Zipper noise** | slider çekerken "merdiven" sesi | tüm gain/pan/cutoff'u 5–20 ms yumuşat; log-domain'de interpolasyon |
| **Denormals** | CPU aniden %100, ses kesilir | FTZ/DAZ; reverb/comb feedback yollarında offset |
| **Sample-rate bağımlı sabit** | 44.1 kHz'de doğru, 48 kHz'de yanlış tuning | tüm delay uzunlukları `fs/44100` ile ölçekle; katsayılar `prepare`'da |
| **Faz süreksizliği** | transient başında "tık", loop dikişinde "vın" | osilatör reset'te 0.5–2 ms fade; grain window; loop uzunluğunu periyoda hizala |
| **Float32 modal hassasiyeti** | düşük frekanslı uzun modlarda "cıvıldama" | `f < 150 Hz` veya `T60 > 1 s` için `double`; ya da coupled form |
| **Modal toplam clipping** | çok modlu bankta tepe > 1.0 | `sum g_k` değil, ölçülmüş IR enerjisiyle normalize et; çıkışta yumuşak limiter |
| **Reverb DC birikmesi** | kuyruk zamanla "şişer", headroom kaybı | comb/allpass içinde 20 Hz HP |
| **Granular overlap hatası** | density değişince seviye zıplar | `1/sqrt(overlap)` normalizasyonu |
| **Convolution latency** | kuru ses ile ıslak ses arasında kayma | `pre_delay` ve partition gecikmesini telafi et |
| **Dither yanlış yerde** | 24-bit'te gereksiz gürültü; 16-bit'te yok | sadece final bit-depth düşüşünde TPDF |
| **Determinizm kaybı** | aynı seed farklı sonuç | `-ffast-math` kapalı; FMA varsa sabit; thread sayısından bağımsız toplama sırası |

---

## 9. Kritik Node'lar için Pseudo-code

### 9.1 `ModalBank` — SoA resonator bankası

```cpp
struct ModalBank {
    static constexpr int MAX_MODES = 64;
    float a1[MAX_MODES], a2[MAX_MODES], g[MAX_MODES];
    float y1[MAX_MODES], y2[MAX_MODES];
    int   n_modes = 0;

    void set_mode(int k, float f, float t60, float gain_db, float fs) {
        const float r  = std::exp(-6.907755278f / (t60 * fs));   // T60 -> pole radius
        const float th = 2.0f * float(M_PI) * f / fs;
        a1[k] = 2.0f * r * std::cos(th);
        a2[k] = -r * r;
        g[k]  = std::sqrt(1.0f - r * r) * std::pow(10.0f, gain_db / 20.0f);
        y1[k] = y2[k] = 0.0f;
    }

    // block-process, branch-free; auto-vectorizable
    void process(const float* __restrict x, float* __restrict out, int n) {
        for (int k = 0; k < n_modes; ++k) {
            const float b1 = a1[k], b2 = a2[k], gg = g[k];
            float s1 = y1[k], s2 = y2[k];
            for (int i = 0; i < n; ++i) {
                const float y = b1 * s1 + b2 * s2 + gg * x[i];
                s2 = s1; s1 = y;
                out[i] += y;
            }
            y1[k] = s1; y2[k] = s2;
        }
    }
};
```

**Not:** iç döngü `k` üzerinden dış, `i` üzerinden iç → her mod kendi state'ini taşır, `out[i] +=` birikimlidir.
AVX2 için `k` 8'erli gruplara ayrılır (`a1[8]` yüklenir, `s1/s2` vektörlenir), `i` döngüsü skaler kalır.

### 9.2 `TransientGen` — darbe/click

```cpp
// 0.2..10 ms darbe; curve=4 => "snappy", curve=1 => lineer
float transient_sample(float t, float dur, float curve, float phase) {
    if (t >= dur) return 0.0f;
    const float u = t / dur;
    const float env = std::pow(1.0f - u, curve);        // power decay
    const float w = 2.0f * float(M_PI) * 3200.0f * t + phase;  // taşıyıcı ~3.2 kHz
    return env * (0.7f * std::sin(w) + 0.3f * (2.0f * rand_bipolar() ));  // ton + gürültü
}
```
Daha yüksek kalite: `env * noise` → BPF; düşük frekanslı "gövde" ayrı katman.

### 9.3 `TunedNoiseBand` — TPT SVF + envelope

```cpp
struct TunedNoise {
    float g, k, a1, a2, a3, ic1 = 0, ic2 = 0;
    void set(float fc, float q, float fs) {
        fc = std::clamp(fc, 20.0f, 0.45f * fs);
        g  = std::tan(float(M_PI) * fc / fs);
        k  = 1.0f / std::max(q, 0.05f);
        a1 = 1.0f / (1.0f + g * (g + k));
        a2 = g * a1;
        a3 = g * a2;
    }
    float process(float x) {
        const float v3 = x - ic2;
        const float v1 = a1 * ic1 + a2 * v3;
        const float v2 = ic2 + a2 * ic1 + a3 * v3;
        ic1 = 2.0f * v1 - ic1;
        ic2 = 2.0f * v2 - ic2;
        return v1;                       // band-pass
    }
};
```

### 9.4 `GrainCloud` — zamanlayıcı + overlap-add

```cpp
struct Grain { double pos; float rate; float pan; int len; int age; float amp; };

void GrainCloud::render(float* outL, float* outR, int n, double sr) {
    double t = 0.0;
    while (t < n) {
        // deterministic grid + jitter (uniform), Poisson alternatifi: -log(u)/density
        const double period = 1.0 / density;
        const double jit = (rng.next_float() * 2.0 - 1.0) * period * jitter_amount;
        const double onset = std::max(0.0, t + jit);
        spawn_grain(onset);
        t += period;
    }
    // overlap-add
    for (auto& gr : active) {
        for (int i = gr.age; i < gr.len && i - gr.age < n; ++i) {
            const float w = hann((float)i / gr.len);
            const float s = cubic_interp(source, gr.pos) * w * gr.amp;
            const float panL = std::cos((gr.pan + 1) * 0.25f * M_PI);
            const float panR = std::sin((gr.pan + 1) * 0.25f * M_PI);
            outL[i - gr.age] += s * panL;
            outR[i - gr.age] += s * panR;
            gr.pos += gr.rate * (1.0 + pitch_jitter * rng.next_float());
        }
        gr.age += n;
        if (gr.age >= gr.len) gr.dead = true;
    }
    // normalize: gain = 1.0f / std::sqrt(std::max(1.0f, density * grain_ms * 0.001f));
}
```
**Dikkat:** `spawn_grain` ses thread'inde **allocation yapmaz**; `active` fixed-size array + serbest liste.

### 9.5 `Rotor` — blade-pass AM + drift

```cpp
struct Rotor {
    float rpm, blades, depth, drift;
    double phase = 0.0;          // 0..1, BPF periyodu
    void process(float* out, int n, double sr, float noise_bp, float lfo) {
        const float bpf = (rpm / 60.0f) * blades;
        for (int i = 0; i < n; ++i) {
            const float am = 1.0f - depth * 0.5f * (1.0f - std::cos(2.0f * M_PI * float(phase)));
            const float drift_gain = 1.0f + drift * 0.01f * lfo;     // lfo ∈ [-1,1], 0.05..0.3 Hz
            // noise_bp: zaten BPF'ten geçmiş gürültü (bpf merkezli)
            out[i] += noise_bp * am * drift_gain;
            phase += bpf / sr;
            if (phase >= 1.0) phase -= 1.0;
        }
    }
};
```
Harmonikler için `bpf` yerine `n*bpf` (n=1..5) ve her biri için ayrı faz akümülatörü; kazanç `1/n^0.9`.

---

## 10. Sonraki Adımlar (bu dokümandan çıkan işler)

1. `packages/dsp` içinde yukarıdaki 15 node'un C++ header-only implementasyonu + unit test (impulse response doğrulama).
2. Her kategori için `recipes/*.json` — §3'teki değerlerin birebir veri olarak yazılması.
3. `style` sürücü tablosunun (§4) koda `struct StyleDrivers` olarak aktarılması; UI'da tek slider + advanced panel.
4. `memo`/seed altyapısı: SplitMix64 + LHS family generator.
5. LUFS/true-peak ölçüm modülü (BS.1770 uyumlu) ve export zinciri.
6. Benchmark harness: 32 voice, 16 modal bank, 4 granüler cloud ile CPU ölçümü.

---

## 11. Kaynaklar (fiilen açılan URL'ler)

- Julius O. Smith III, *Physical Audio Signal Processing* — <https://ccrma.stanford.edu/~jos/pasp/pasp.html>
- Aynı kitap, "State Space to Modal Synthesis" — <https://ccrma.stanford.edu/~jos/pasp/State_Space_Modal_Synthesis.html>
- Aynı kitap, "Freeverb" (Schroeder-Moorer yapısı, stereospread=23) — <https://ccrma.stanford.edu/~jos/pasp/Freeverb.html>
- W3C, *Audio EQ Cookbook* (RBJ biquad formülleri) — <https://www.w3.org/TR/audio-eq-cookbook/>
- musicdsp.org, "State variable" (Chamberlin SVF, Dattorro) — <https://www.musicdsp.org/en/latest/Filters/23-state-variable.html>
- Robin Whittle, *DSP generation of Pink (1/f) Noise* (Kellet filtresi, Voss-McCartney) — <https://www.firstpr.com.au/dsp/pink-noise/>
- Fagerström, Schlecht, Välimäki, *Non-Exponential Reverberation Modeling Using Dark Velvet Noise*, arXiv:2403.20090 —
  <https://arxiv.org/html/2403.20090v1> (velvet noise yoğunluğu 1500–2000 pulse/s)
- EBU R 128 (LUFS, gating, true-peak) — <https://en.wikipedia.org/wiki/EBU_R_128>
- Blast wave / Friedlander waveform — <https://en.wikipedia.org/wiki/Blast_wave>

**Açılamayan ama standart referans olarak kullanılanlar** (PDF/erişim engeli nedeniyle doğrulanamadı, kod yazılırken
tekrar kontrol edilmeli): Hacıhabiboğlu, "Procedural Synthesis of Gunshot Sounds Based on Physically Motivated Models"
(Springer, 2017); Andy Farnell, *Designing Sound* (MIT Press, 2010); Curtis Roads, *Microsound* (MIT Press, 2001).
