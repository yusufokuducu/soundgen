# 02 — Araç Teardown'u ve SoundGen UX/Ürün Spesifikasyonu

> Kapsam: Profesyonel sound design / synthesis araçlarının nasıl *hissettirdiğini* tersine mühendislikle çıkarmak
> ve bundan SoundGen için uygulanabilir bir arayüz sözleşmesi üretmek.
> Yöntem: üretici dokümanları, bağımsız review'lar, resmî manual'lar ve **gerçek ekran görüntüleri** (Enrage, Reformer Pro,
> Weaponiser, Phase Plant) üzerinden görsel teardown. Kaynaklar §10'da.
>
> Bu doküman bir *görüş* dokümanıdır. "Şu da olabilir" değil, "şu olacak, çünkü..." yazar.

---

## 0. Üç cümlelik özet

Profesyonel SFX araçlarının hepsi aynı üç şeyi çözmeye çalışıyor: **(1) tuşa basınca ses çıksın**, **(2) sesi tek jestle
değiştirebileyim**, **(3) çıkanı oyuna sokabileyim**. Krotos bunu *performans* üzerinden (XY pad, FIRE tuşu), Boom *DSP
derinliği* üzerinden (rack + macro), synth'ler *modülasyon görselleştirmesi* üzerinden çözüyor. Hiçbiri **"aile"**
kavramını birinci sınıf vatandaş yapmıyor: hepsi tek bir sesi mükemmelleştiriyor, oyunun ihtiyacı olan *tekrar etmeyen
kümeyi* kullanıcıya elle kurduruyor. SoundGen'in boşluğu tam burada: **seed + morph + family export** tek akışta.

---

## 1. Competitive teardown

### 1.1 SFX / performans araçları

| Araç | Çekirdek metafor | Real-time audition | Randomization | Macro | Export | Kullanıcıların sevdiği / şikâyet ettiği |
|---|---|---|---|---|---|---|
| **Krotos Reformer Pro** | 4 kütüphanenin **XY pad** üzerinde morph'u; ses girişiyle tetikleme | Girdi sinyali (mikrofon/DAW) canlı analiz edilir, transient + spektrum + genlik'e göre sample seçilir. Gecikme hissedilmez çünkü "enstrüman" gibi çalınır | `Variation` slider'ı algoritmanın seçimden "roam" etmesini sağlar — tekrarı kıran tek knob | 4 voice × (Volume, Speed, Response) + Transient Engine; Master compressor/limiter | Mono çıkış; `Reformer Analysis Tool` ile kendi kütüphaneni paketleme | **+** "foley'yi gerçekten çalıyorsun", kendi sample'larını import etmek çok kolay. **−** mono; hangi sample'ın çaldığı UI'da görünmüyor; melodic değil |
| **Krotos Weaponiser** | **4 engine × 4 slot = 16 katman**, renk kodlu stack + ortak timeline | Kırmızı **FIRE** butonu; `BURST MODE`, `TAIL MODE`, `FIRE RATE` ile oyun içi his simülasyonu | Her slot 5 sample'ı **cycle** eder → varyasyon otomatik; `DRUNK` slider'ı kasıtlı düzensizlik üretir | "Modular macro system" + parametre-kilitli FX sequencer | Preset kit/pattern; katman başına insert chain + EQ/comp/limiter | **+** renk kodu sayesinde "hangi katman neydi" anında anlaşılıyor; tag'li library tarayıcı. **−** 16 katman sınırı; preview'ı durdurmanın doğal yolu yok; bounce edip yeniden başlamak gerekiyor |
| **Krotos Studio** | **Preset + XY pad performansı**; kullanıcı "ses tasarlamıyor, çalıyor" | Arka planda **sürekli kayıt**; tweak ederken çalan her şey yakalanır, `Drag Recording` ile DAW'a sürüklenir | Preset içinde gizli (kullanıcıya açık değil) | XY pad eksenleri (intensity, distance, near/far, wind/stream) | Drag-and-drop audio; 48 kHz/24-bit | **+** "15 saniyede kullanılabilir ses"; kaydı geri alıp sürüklemek dahice. **−** derinlik yok, abonelik, içerik gelmezse değersiz |
| **Boom Library Enrage** | **Modüler rack** (6 paralel × 8 seri) + matrix + 6 macro | Rack içinde her device canlı; oversampling toggle | Mod source olarak LFO/S&H; formülle modülasyon | 6 macro — **DAW automation'a açık tek şey** | Preset; standalone yok, plug-in | **+** "bilimsel laboratuvar" hissi, preset'ler inanılmaz. **−** öğrenme eğrisi dik; automation için macro'ya mahkûm; undo'ya rağmen "kayboldum" |
| **Boom Library Turbine** | **Fiziksel model** (N1/N2/N3 + combustion), tek büyük `THRUST` kolu | Kol çekilince RPM değişir; simulation sekmesi 360° pozisyon | `LIFE` knob'u modülasyon derinliğini = doğal düzensizliği ayarlar | 8 **bipolar** karakter knob'u (−1..+1): Burn, Angle, Bass, Character, Harmonics, Drive, Life, Spread | Preset bazlı; loop üretimi için tasarlanmış | **+** "gerçek motor" hissi; bipolar knob'lar tek bakışta anlaşılıyor. **−** sadece turbine; başka bir şeye uyarlamak zor |
| **Soundly** | **Arama + metadata + spot-to-DAW** | Liste üzerinde tek tıkla preview; `Shift+Enter`; scrub | Yok (library) | Yok | `spot-to` Pro Tools'a kopyalayarak gönderir; drag-and-drop; **preview'daki pitch/reverse export'a uygulanır** | **+** arama hızı, UCS metadata, "duyduğun gibi export edilir". **−** üretmez, sadece bulur |

### 1.2 Synth / modular araçlar (görsel dil için)

| Araç | Çekirdek metafor | Audition | Randomization | Macro | Export | Övgü / şikâyet |
|---|---|---|---|---|---|---|
| **Phase Plant** | Tek pencerede **generator stack + snapin lane + modülasyon lane** | Her modül kendi mini scope'unu çizer; modül kapatılabilir → "bunsuz nasıl?" testi | Chaos modülatörü; kullanıcı bazlı | **8 büyük macro knob**, tek macro birden çok hedefe | Patch; audio render DAW'da | **+** "tek pencere, sayfa yok" disiplini; 32 modül. **−** boş ekranla açılıyor, ilk 10 dakika kayıp |
| **Vital** | **Görsel modülasyon**: her bağlantı ekranda canlı | Drag ederken **hedefin üzerine gelince modülasyonu duyurur**, bırakınca bağlar | 4 random modulator: Perlin, S&H, sine-interp, Lorenz | Yok (macro yerine modülasyon) | Preset | **+** "commit etmeden dene" → en iyi UX fikri. **−** 2-3 routing sonrası ekran "noel ağacı" |
| **Arturia Pigments** | **6 engine + renk kodlu modülasyon halkaları**; `Play` vs `Synth` görünümü | Play view: 8 macro + FX, sesi bozmadan | Random generator'lar | 4 macro, başka modülatörlerle modüle edilebilir | Preset | **+** ilerlemeli açıklık (progressive disclosure) ders kitabı gibi; renkli halkalar derinliği bir bakışta veriyor. **−** 6 engine = CPU; "her şeyi yapan, hiçbir şeye benzemeyen" |
| **UVI Falcon** | **Hiyerarşi**: oscillator → keygroup → layer → patch → multi | Katman katman solo; Info view'da macro | Script (Lua) ile | Info view macro'ları | Multi/patch | **+** sınırsız derinlik. **−** "kolayca kaybolursun"; renk kodu eksikliği review'larda açıkça eleştirilmiş |
| **Xfer Serum** | Wavetable + **preset hybridize** | Klavye/DAW | `Chaos` + `NoteOn Rand` + **hybridize**: iki preset'in knob değerlerini rastgele karıştır | Matrix | Preset | **+** hybridize = "iki fikirden üçüncüsü". **−** randomize'ın ne yaptığı belirsiz, geri alınabilirliği zayıf |

### 1.3 DAW / teslimat tarafı

- **Reaper**: `Region Render Matrix` + **wildcard** dosya adlandırma (`$region`, `$track`) + render queue. Oyun sesi
  üretiminde fiilî standart, çünkü *çok sayıda küçük dosyayı* isimlendirerek dökmek birinci sınıf özellik.
- **Ableton Live**: `Q` hot-swap (tarayıcıda gezerek cihaz değiştirme), `P` **A/B compare**, `Shift+Enter` preview,
  ok tuşlarıyla listede gezinme. "Klavyeden ayrılmadan tarayıcı"nın referansı.
- **Wwise / FMOD**: `Random Container` + `Shuffle` + **"Avoid repeating last X played"**; FMOD'da playlist
  "shuffle no-repeat" ve global sequential mod. Yani oyun tarafındaki tekrar önleme *çözülmüş* bir problem — ama
  ses *tasarımı* aracı bu kümeyi üretmiyorsa çözüm işe yaramıyor.

### 1.4 Teardown'dan çıkan 6 sert kural

1. **Tek bir "çal" jesti olmadan hiçbir araç kullanılmıyor.** Weaponiser'ın FIRE'ı, Studio'nun pad'i, Reformer'ın
   input'u — hepsi aynı şey: *denemek = çalmak*.
2. **Renk, kimliğin taşıyıcısı.** Weaponiser 4 engine'i renklendirdiği için 16 katman okunabiliyor. Falcon
   renklendirmediği için kayboluyor. Renk süs değil, navigasyon.
3. **Derinlik ilerlemeli olmalı.** Pigments'in `Play/Synth` ayrımı, Falcon'un `Info` view'ı — aynı çözümün iyi ve kötü
   hâli. SoundGen'de üç seviye: Preset → Generator → Layer.
4. **Randomize, kilitsiz olamaz.** Serum'un hybridize'ı ve Enrage'ın formül modülasyonu "ne oldu bilmiyorum" hissi
   üretiyor. Kilit (lock) olmadan randomize = kumar.
5. **Export bir diyalog değil, bir akış.** Reaper'ın wildcard'ları ve Soundly'nin spot-to'su kazanıyor; "File > Export >
   ayarlar > tamam" kaybediyor.
6. **Aşırı modülerlik SFX'te kayıp.** Enrage ve Falcon'ın şikâyeti aynı: güç var, yön yok. SoundGen'in kullanıcısı
   *ses tasarımcısı değil, oyun geliştiricisi* — node graph vermeyeceğiz.

---

## 2. Kopyalanacak kazanan interaction pattern'ler

Aşağıdakiler "ilham" değil, **spec**. Her biri SoundGen'de nasıl uygulanacağıyla yazıldı.

| # | Pattern | Kimden | SoundGen'de |
|---|---|---|---|
| 1 | **One-key audition** | Weaponiser `FIRE`, Studio pad | `Space` = play/stop, `Enter` = retrigger. Playhead durmaz; her parametre değişimi bir sonraki tetikte duyulur. |
| 2 | **Audition-on-change** | Reaper/DAW monitoring | Tek-shot'larda varsayılan **açık**: knob bırakılınca 120 ms içinde otomatik tetikleme. Loop'larda kapalı (kesintisiz dinleme). |
| 3 | **Randomize while listening** | Serum hybridize, Enrage | `R` = sadece **kilitli olmayan** parametreleri randomize et ve *çalmaya devam eden* sesin bir sonraki tetiğinde duyur. |
| 4 | **Dice + lock per parameter** | Serum/Vital modülasyon disiplini | Her knob'un sağ üstünde 6 px `◈` (lock) ve altında 8 px `⚄` (dice) hotspot. Kilitliyse dice soluk; `Shift+click` ile kilidi kır. |
| 5 | **Modülasyon önizlemesi** | Vital ("hover to audition, release to commit") | Morph slider'ı sürüklerken ses **anında** değişir (tek render, parametre morph'u — §4); bırakmadan `Esc` = geri dön. |
| 6 | **Macro / bipolar knob** | Turbine (−1..+1), Phase Plant (8 macro) | Morph dışında 4 sabit macro: `Size`, `Distance`, `Brightness`, `Wildness`. Hepsi bipolar, hepsi kilitlenebilir. |
| 7 | **A/B compare, level-matched** | Ableton `P` | `A`/`B`: iki tam state. Geçiş **10 ms crossfade + LUFS eşitleme** ile — yoksa "daha yüksek olan daha iyi" yanılgısı. |
| 8 | **Undo, jest bazlı** | Enrage, Phase Plant | Bir drag = bir undo. Etiketler anlamsal: `Randomize Tail (seed 8831)`, `Morph 0.42`. `Ctrl+Alt+Z` ile **history scrub** (görsel + işitsel). |
| 9 | **Waveform'lu varyant tarayıcı** | Soundly, Weaponiser library | Variation grid'de her hücrede mini waveform + seed + LUFS. Hover = 300 ms'lik otomatik audition. |
| 10 | **Drag-to-DAW export** | Soundly spot-to, Studio drag recording | Herhangi bir varyant hücresini **Explorer/DAW penceresine sürükle** → geçici 48 kHz WAV üretilir, `CF_HDROP` ile bırakılır. |
| 11 | **Sürekli yakalama tamponu** | Studio background record | Son **90 saniye** her zaman ring buffer'da. `Ctrl+Shift+C` = "az önce beğendiğim şeyi kurtar". |
| 12 | **Kapat → "bunsuz nasıl?"** | Phase Plant modül bypass | Her layer satırında `⏻`; solo/mute değil, *bypass* (parametreleri koruyarak). |

---

## 3. Information architecture

### 3.1 Global iskelet

Altı ekran, **tek kalıcı transport**. Ekran değişince ses ve playhead kaybolmaz — bu, DAW'lardan ödünç alınan ve
SFX araçlarının çoğunda eksik olan en önemli karar.

```
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ SoundGen        Weapon › Pistol › Fire   sfx_wpn_pistol_fire_03    seed 871   Real ━●━ Toon  │
│ 48 px           preset yolu (breadcrumb) isim  (F2 = rename)          ⚄ dice    morph 280 px │
├───────┬────────────────────────────────────────────────────────────────┬─────────────────────┤
│ NAV   │   GENERATOR                                        [Perform]   │ INSPECTOR           │
│ 56px  │                                                                │ 320 px              │
│ [P]   │  ┌──────────────────────────────────────────────────────────┐  │ Param               │
│ [G]   │  │ ● Transient    gain  pan  pitch  decay    ⏻ ⚄ ◈   ▁▃▅▂   │  │ Body Pitch          │
│ [L]   │  └──────────────────────────────────────────────────────────┘  │    +142 Hz          │
│ [T]   │  ┌──────────────────────────────────────────────────────────┐  │ ┌───────────────┐   │
│ [V]   │  │ ● Body         gain  pan  pitch  decay    ⏻ ⚄ ◈   ▁▅▇▃   │  │ │ curve editor  │   │
│ [E]   │  └──────────────────────────────────────────────────────────┘  │ └───────────────┘   │
│ [⚙]   │  ┌──────────────────────────────────────────────────────────┐  │ Lock  ◈             │
│       │  │ ● Tail         gain  pan  pitch  decay    ⏻ ⚄ ◈   ▁▂▃▁   │  │ Bias  +0.40         │
│       │  └──────────────────────────────────────────────────────────┘  │                     │
│       │  ┌──────────────────────────────────────────────────────────┐  │ ── LAYERS ─────     │
│       │  │ ● Mechanism    gain  pan  pitch  decay    ⏻ ⚄ ◈   ▁▁▂▁   │  │ ● Transient         │
│       │  └──────────────────────────────────────────────────────────┘  │ ● Body              │
│       │                          [+ Add layer]  [+ Add FX]             │ ● Tail              │
│       │                                                                │ ● Mechanism         │
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│ ▶ ███████████████████████████▏      0.42s / 1.80s        [loop ⟲]   [1.0×]                   │
│   waveform + playhead + katman renk şeritleri                    96 px                       │
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│ PEAK −6.2 │ TRUE PEAK −1.0 │ RMS −18.4 │ M −14.1 │ S −14.8 │ I −15.0 │ 3.1 ms                │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

Rail ikonları: `[P]` Presets, `[G]` Generator, `[L]` Layers, `[T]` Timeline & FX, `[V]` Variations,
`[E]` Export, `[⚙]` Settings. Aktif ekran `#1C232C` zemin + 3 px `#4CC2FF` sol çubukla işaretlenir.

**Bölge sözleşmeleri**

- **Top bar (48 px)**: her zaman morph slider'ı barındırır. Ürünün tek "marka" kontrolü sürekli görünür olmalı.
- **Left rail (56 px)**: ikon + 11 px uppercase label; aktif olan 3 px accent çubuğu ve `#1C232C` arka plan.
- **Center**: ekranın kendisi. Minimum 720 px genişlik.
- **Inspector (320 px)**: tek parametreye odaklanır; "detay" hissi buradan gelir. `Ctrl+I` ile kapanır.
- **Transport (96 px)**: dalga formu + playhead + katman şeritleri. **Tüm ekranlarda aynı.**
- **Meter bar (28 px)**: peak/true-peak/RMS/LUFS M-S-I + sample rate + **gerçek latency**.

### 3.2 Ekranlar

**1) Preset Browser** — Sol: kategori ağacı (Weapons / Zombies / Gore / Foley / Ambience / Loops / Crate / Confetti /
UI / Whoosh). Orta: kart grid'i (3 sütun, 240×120 px kart; mini waveform, isim, süre, LUFS, 3 tag). Sağ: seçili
preset'in büyük waveform'u + "Open". Hover = otomatik audition (300 ms sonra, loop'suz). `↑/↓` gezinme,
`Shift+Enter` preview, `Enter` aç. Üstte arama (`Ctrl+F`), tag filtreleri ve **"son kullanılanlar"**.

**2) Generator View (ana ekran)** — Bu, preset'in *reçetesi*. Ortada "generator kartları" dikey akışta:
`Transient` → `Body` → `Tail` → `Mechanism` (silah için) gibi. Her kart 88 px yüksekliğinde, tek satırda 5-6 kontrol,
sağda mini scope (Phase Plant disiplini). Kart başlığında renk noktası + isim + `⏻` bypass + `⚄` dice + `◈` lock.
Sağ üstte `[Perform]` sekmesi: X = morph, Y = size olan **XY pad** (280×280 px).

**3) Layer Editor** — Weaponiser'ın stack'i, ama satır bazlı: her satır 64 px; solda 4 px renk şeridi, katman adı,
waveform (drag = trim), `Gain / Pan / Pitch / Offset / Decay` mini slider'ları, `M/S/B` ve `⚄`. Üstte "Add layer"
(6 tip: `Tonal`, `Noise`, `Transient`, `Granular`, `Sample`, `Synth`). Sürükle-bırak sıralama; `Ctrl+G` grup.

**4) Timeline & FX** — X ekseni zaman (ms), Y ekseni katman. Her katman bir blok; blok kenarından sürükleyerek
`offset` ve `duration`. Blok üzerinde **envelope** (ADSR + eğri) doğrudan düzenlenir. Alt panel: ortak FX zinciri
(6 slot: `EQ`, `Sat`, `Comp`, `Transient`, `Reverb`, `Limiter`). Loop modunda `in/out` + `crossfade` (0-500 ms).

**5) Variation Grid** — 20 hücre, 5×4. Hücre 200×112 px: mini waveform, seed, LUFS, `K`/`X` durumu. Üstte
`Generate 20` + `Spread` slider'ı (0-100) + `Distinctness` göstergesi.

**6) Export** — Tek sayfa, üç sütun: **Naming** (canlı önizleme), **Audio** (48 kHz, mono/stereo, bit depth, normalize
hedefi), **Delivery** (klasör, manifest, Unity `Resources` kopyası). Alt sağda `Export` + `Export & Reveal`.

### 3.3 Varsayılan keybind'ler (Windows-first, `Ctrl`)

| Kapsam | Tuş | İşlev |
|---|---|---|
| Global | `Space` | Play / Stop (loop-aware) |
| Global | `Enter` | One-shot tetikle (playhead sabit) |
| Global | `Shift+Enter` | Yeni seed ile tetikle (kaydetmeden) |
| Global | `Esc` | Tüm voice'ları 5 ms fade ile kes |
| Global | `Ctrl+1..6` | Preset / Generator / Layers / Timeline / Variations / Export |
| Global | `Ctrl+B` / `Ctrl+I` | Browser rail / Inspector toggle |
| Global | `Ctrl+Z` / `Ctrl+Shift+Z` | Undo / Redo |
| Global | `Ctrl+Alt+Z` | History panel (scrub) |
| Global | `Ctrl+S` / `Ctrl+Shift+S` | Save / Save as |
| Morph | `M` | Morph slider'a odaklan |
| Morph | `←` / `→` | ±0.05 (`Shift` ile ±0.01) |
| Morph | `0` | Merkeze dön (0.0) |
| Morph | `Shift+drag` | Fine (0.1×) |
| Morph | `Alt+click` | Sıfırla |
| A/B | `A` / `B` | Snapshot'a geç (level-matched) |
| A/B | `Shift+A` / `Shift+B` | Snapshot'a yaz |
| Random | `R` | Kilitli olmayanları randomize et |
| Random | `Shift+R` | Kilitleri yok say, hepsini randomize et |
| Random | `Ctrl+R` | Sadece seed'i yenile |
| Random | `L` | Hover/selection'daki parametreyi kilitle |
| Random | `Shift+L` / `Alt+L` | Hepsini kilitle / hepsini aç |
| Layer | `Alt+1..8` | Katman seç |
| Layer | `Shift+Alt+1..8` | Solo |
| Layer | `Ctrl+Alt+1..8` | Mute |
| Layer | `Ctrl+D` / `Delete` | Çoğalt / sil (undo'lu) |
| Timeline | `I` / `O` / `H` | Loop in / out / toggle |
| Timeline | `F` | Fit view |
| Timeline | `W` | Waveform ↔ Spectrogram |
| Var. | `V` | Variation Grid |
| Var. | `G` | Generate (20) |
| Var. | `1..0` / `Shift+1..0` | 1-10 / 11-20 audition |
| Var. | `K` / `X` / `U` | Keep / Discard / Un-keep |
| Var. | `C` | Seçiliyi mevcut sesle karşılaştır |
| Export | `Ctrl+E` / `Ctrl+Shift+E` | Export / Export family |
| Export | `Ctrl+Enter` | Export & Reveal |

**Knob modifiers (tek standart, istisnasız):** dikey sürükle = kaba; `Shift` = 0.1×; `Ctrl` veya çift tık = numerik
giriş (`-6.0 dB`, `1200 ms`, `0.5`); `Alt` = default; tekerlek = tek adım; sağ tık = `Lock / Randomize / Reset /
Automate`. Tam aralık **200 px** dikey mesafe — çözünürlükten bağımsız hassasiyet.

---

## 4. "Realism ↔ Cartoon" kontrolü

### 4.1 Öneri (net)

**Tek bipolar slider, top bar'da sabit, 280 px, ortada detent (0.0), uçlarda `Real` / `Toon` etiketi.**
Yanında `Perform` görünümünde **ikinci eksen olarak `Size`** olan bir XY pad — ama *sadece* Generator ekranında,
çünkü Scale orada fiziksel bir anlam taşıyor (whoosh'un boyu, patlamanın ölçeği).

Yani: **ana kontrol = tek eksen; ikinci eksen = bağlama özel, opsiyonel.**

### 4.2 Neden tek slider (audio-UX gerekçesi)

- **Tek boyut, tek jest.** Hick's law: seçenek sayısı arttıkça karar süresi artar. Ses tasarımında kullanıcı zaten
  20 parametreyle boğuşuyor; stil ekseni *tek* olmalı ki "bunu biraz daha çizgi film yap" cümlesi tek harekete dönüşsün.
- **Söylenebilir olması.** Tek eksen, ekibin diline girer: "bu ses %70 cartoon". XY pad'de böyle bir cümle kurulamaz.
- **Fitts' law.** Top bar'daki 280×24 px hedef, ekranın her yerinden erişilebilir; 20 px'lik bir XY pad köşesinden
  çok daha hızlı. Morph günde 200 kez kullanılacak bir kontrol — hedef alanı büyük olmalı.
- **Görsel geri bildirim eşleşmesi.** Slider'ın arkasındaki "hayalet waveform" (real → toon silueti) tek eksende
  okunabilir; XY'de iki boyutlu bir bulanıklığa dönüşür.
- **Yanlış kullanım maliyeti.** XY pad'de kullanıcı farkında olmadan ikinci ekseni kaydırıp "ses bozuldu" der ve
  sebebini bulamaz. Tek eksende böyle bir tuzak yok.

XY pad yine değerli: Krotos Studio ve Reformer'da XY, *performans* için var (aynı anda iki şeyi çalmak). SoundGen'de
performans kaydı MVP değil; bu yüzden XY ikincil.

### 4.3 Morph nasıl uygulanır: crossfade değil, **parameter morph**

İki ayrı render'ı crossfade etmek cazip görünür ama üç nedenden dolayı yanlıştır:

1. **Comb filtering.** İki render aynı kaynağın farklı işlenmiş hâliyse yüksek oranda korelidir; lineer crossfade
   faz farkı olan bölgelerde tarak filtre üretir. Equal-power crossfade bunu çözmez, sadece maskeler.
2. **Çift transient (flam).** Real render'ın 0.5 ms'lik attack'ı ile cartoon render'ın 2 ms'lik "boing" attack'ı
   üst üste binerse duyulan şey tek bir vuruş değil, iki vuruştur. SFX'te bu ölümcül.
3. **Tail uyumsuzluğu.** Crossfade sırasında iki farklı decay eğrisi toplanır → enerji şişmesi, sonra çöküş.

**Doğru yol:** tek render, parametrelerin sürekli interpolasyonu. `morph ∈ [0,1]` bir "parameter map"i sürer:

| Parametre | Real (0.0) | Cartoon (1.0) | Not |
|---|---|---|---|
| Attack time | 0.4 ms | 1.8 ms | 0.3'ten sonra devreye girer |
| Transient overshoot | 1.0× | 1.35× | "smack" hissi |
| Spectral tilt | 0 dB | +6 dB @ 3 kHz | "smile" eğrisi |
| Body resonance Q | 12 | 3.5 | rezonans kaybı = plastiklik |
| Body pitch drop | 0 st | −7 st / 60 ms | en güçlü cartoon ipucu |
| Noise / tonal oranı | 0.80 | 0.35 | tonalite artar |
| Saturation | 0 dB | +6 dB (tanh) | yumuşak clip |
| Tail uzunluğu | 1.0× | 1.5× | loop'ta periyoda yuvarlanır |
| Dynamics | korunur | sıkıştırılmış | "bonk" |
| Sub (60 Hz) | −6 dB | +2 dB | ağırlık |
| Kuru/ıslak | %18 reverb | %0 | cartoon kuru olur |

**Kritik detay — ease window'lar.** Her parametre [0,1] aralığının tamamında hareket etmez; her birinin bir
`window = [start, end]` değeri vardır. Örn. `tail_length` ancak 0.45'ten sonra değişmeye başlar. Bu olmadan morph
"her şey aynı anda kayıyor" hissi verir ve ara değerler çamurlaşır. Bu, ürünün en çok emek isteyen ama en çok
fark yaratan yeridir.

**Ayrık eşikler.** 0.70'in üzerinde `zip`/`boing` katmanı (pitch-envelope'lu kısa sinüs) devreye girer; 0.85'in
üzerinde reverb tamamen kapanır. Bunlar sürekli interpolasyon değil, *kademeli reveal* — ama 30 ms fade ile
girerlerse duyulmaz, sadece "zenginleşme" olarak algılanır.

**Per-layer bias.** Her katmanın `morphBias ∈ [−1,+1]` değeri vardır. `Transient` katmanı −0.3 (geç cartoon'laşır),
`Tail` katmanı +0.4 (erken cartoon'laşır). Böylece global slider tek jest olarak kalır ama sonuç tek düze olmaz.
Bu, "tek eksen" kararının sakıncasını ortadan kaldırır.

**Per-parameter lock.** Kullanıcı `Body pitch`'i kilitleyip geri kalanı cartoon'a çekebilir. Kilit, morph'un
*hesaplanmasında* devre dışı bırakır (render sonrası değil) — yani phase/timing sorunu doğmaz.

### 4.4 Değişimi anında görme ve duyma

- Slider sürüklenirken **ses kesintisiz** kalır: mevcut voice etkilenmez, bir sonraki tetik yeni morph ile gelir.
  Tek-shot'ta bu 60-120 ms içinde olur (audition-on-change), yani "anında" hissi korunur.
- Loop'larda morph değişimi **250 ms equal-power parametre ramp** ile uygulanır (gain değil, parametre).
- Slider'ın arkasında **ghost waveform**: real silueti `#2A323D`, toon silueti `#3A4654`; mevcut morph ikisinin
  arasında interpolasyonla çizilir. Kullanıcı *duymadan önce* ne olacağını görür.
- Sağda **Δ metre**: `LUFS Δ`, `Length Δ`, `Brightness Δ`. Morph'un yan etkilerini sayı olarak göster — sürpriz olmasın.
- **Morph audit** (geliştirici aracı): 0 / 0.25 / 0.5 / 0.75 / 1.0 için 5 render + spectrogram şeridi. Amaç
  *monotonluk*: dinleyici sıralayabilmeli. Sıralayamıyorsa morph haritası yanlış.

### 4.5 Loop'larda özel durum

Pervane/drone loop'larında morph **loop periyodunu değiştiremez**. `tail_length` gibi parametreler loop uzunluğunun
tam katına quantize edilir; aksi hâlde dikişte tık oluşur. Crossfade bölgesi morph'tan bağımsız sabit kalır.

---

## 5. Real-time görselleştirme gereksinimleri

| Görsel | Spesifikasyon | Neden |
|---|---|---|
| **Waveform + playhead** | Min/max peak per pixel + üstünde RMS zarfı (`#4CC2FF` %35 opaklık). Playhead 1 px `#FFFFFF`, arkasında 8 px glow. 60 fps, GPU. | Peak tek başına "ne kadar yüksek" der, RMS "ne kadar dolu" der. SFX'te ikisi birden gerekir. |
| **Katman şeritleri** | Waveform'un altında 4 px yüksekliğinde renkli şeritler (her katman bir renk). | Weaponiser'ın en iyi fikri: "sesin neresi hangi katman" bir bakışta. |
| **Spectrogram** | STFT 1024 / hop 256, log-frekans ekseni, `inferno` kolormap, −90..0 dB. `W` ile toggle. | Gore/zombi gibi geniş bantlı seslerde waveform yalan söyler. |
| **Envelope editörü** | Katman başına ADSR + bezier; noktalar 8 px, hover'da değer balonu. Timeline'da doğrudan. | "Attack'ı 3 ms yap" işi menüye girmeden yapılmalı. |
| **Metreler** | Peak (dBFS), **true-peak** (4× oversample), RMS, **M** (400 ms), **S** (3 s), **I** (gated). Renk: yeşil < −18, sarı −18..−9, kırmızı > −6. Hedef çizgisi kullanıcı tanımlı. | Oyun standardı LUFS; tek sayı yerine üç zaman ölçeği gerekli. |
| **Loop bölgesi** | Sürüklenebilir `in/out`, crossfade uzunluğu (0-500 ms) sayısal + görsel "kelebek" ile. Dikiş noktası kırmızıya dönerse uyarı. | Pervane loop'u dikişte tıklarsa çöp. |
| **Scrub** | Sadece dururken; granular scrub, 40 ms grain, pitch korunur; hız = fare hızı. | "Şuraya bakayım" için play'e basmak zorunda kalmak akışı bozar. |
| **Varyant thumbnail** | 120×36 px mini waveform, seed, LUFS. Keep edilmişse sol kenarda 3 px yeşil çubuk. | 20 varyantı gözle taramak, tek tek dinlemekten hızlı. |

**Performans bütçesi (i5-11400H / 16 GB):** waveform çizimi < 2 ms/frame; STFT < 3 ms/frame (1024 pencere, 48 kHz);
20 varyantın offline render'ı 1.5 s'lik sesler için **< 1.2 s** (6 çekirdek, paralel). Bunlar tutmuyorsa UI "donuyor"
demektir ve tüm ürün yavaş hisseder.

---

## 6. Varyasyon iş akışı (20 varyant, 60 saniye)

### 6.1 Seed ailesi

Her preset bir `seedRoot` taşır. Varyant = `hash(seedRoot, index)` → deterministik. Aynı seed her zaman aynı sesi verir
— bu, "geçen hafta beğendiğim varyantı bul" problemini çözer ve export manifest'ine yazılabilir.

**Spread (0-100)** = varyantların birbirinden ne kadar uzaklaşacağı. 0 = neredeyse ikizler (in-game için ideal değil),
100 = farklı silahlar gibi. Varsayılan **35**.

### 6.2 Üret → dinle → seç (tek ekranda, tek klavyeyle)

1. `G` → 20 varyant üretilir (grid dolar, en fazla 1.2 s).
2. **Auto-audition**: `Shift+Space` ile sıralı dinleme başlar; her varyant 1.2 s + 0.3 s boşluk. Toplam ~30 s.
3. Dinlerken `K` = keep, `X` = discard. Karar vermeden sonraki varyanta geçilir (non-blocking).
4. `1..0` ile istediğin varyanta atla, `C` ile mevcut sesle A/B yap.
5. Keep edilenler üstte **shortlist şeridine** taşınır (yaratıcı akış: grid küçülmez, şerit büyür).
6. `Ctrl+Shift+G`: seçili varyantın **komşuluğunda** 10 yeni varyant (local refine). "Neredeyse doğru"yu bulmanın yolu.
7. `T` = tag (renk + serbest metin + otomatik tag: `close`, `distant`, `metallic`, `wet`).

### 6.3 Distinctness — ürünün gizli silahı

20 varyantın 12'si birbirine benziyorsa kullanıcı 20 değil 8 varyant almıştır. SoundGen her aile için bir
**distinctness matrisi** hesaplar (log-mel veya MFCC mesafesi + süre farkı) ve iki uyarı verir:

- `⚠ Varyant 07 ve 12 neredeyse aynı (0.03)` → biri soluk gösterilir, "regenerate" önerilir.
- `⚠ Aile çeşitliliği düşük (ortalama 0.08)` → `Spread`'i artır önerisi.

**Hedef:** bir ailede **5-8 algısal olarak farklı** üye. 20 farklı dosya değil, 6 gerçekten farklı ses + 14 ince
varyasyon doğru dağılımdır (oyunda nadiren 6'dan fazlasına ihtiyaç var).

### 6.4 "In-game no-repetition"

Oyun tarafında tekrar önleme Wwise/FMOD'da çözülmüş: `Shuffle` + **"Avoid repeating last X played"**, FMOD'da
"shuffle no-repeat" + global sequential. SoundGen'in işi bu kümeyi *doğru* üretmek ve manifest'te tarif etmek.

Export edilen `*.family.json`:

```json
{
  "family": "wpn_pistol_fire",
  "seedRoot": 871,
  "created": "2026-10-24T12:00:00Z",
  "audio": { "sampleRate": 48000, "channels": 1, "bitDepth": 24 },
  "variants": [
    { "file": "sfx_wpn_pistol_fire_01.wav", "seed": 871, "lufsShortTerm": -14.2,
      "truePeak": -1.0, "weight": 1.0, "tags": ["close", "dry"], "distinctness": 1.00 },
    { "file": "sfx_wpn_pistol_fire_02.wav", "seed": 872, "lufsShortTerm": -14.4,
      "truePeak": -1.1, "weight": 1.0, "tags": ["close", "room"], "distinctness": 0.71 }
  ],
  "playback": {
    "mode": "shuffle",
    "avoidRepeatLast": 3,
    "pitchJitterCents": [-40, 40],
    "gainJitterDb": [-1.5, 1.5]
  }
}
```

`avoidRepeatLast = 3`, 5+ varyantlı bir ailede pratikte tekrarı imkânsız kılar; 20 varyantlı ailede ise 3 yeterli
(çünkü aynı ses 4 tetik sonra dönerse zaten fark edilmez). **Asla 0 kullanma** — ardışık iki aynı patlama, oyuncunun
ilk fark ettiği şeydir.

---

## 7. Erişilebilirlik ve ergonomi

### 7.1 Gecikme

| Senaryo | Hedef | Kabul | Not |
|---|---|---|---|
| Tuşa bas → ses | **< 10 ms** | 20-40 ms | Jack ve ark. (2016): 0 ve 10 ms benzer derecelendirildi; 20 ms anlamlı biçimde kötü. |
| Knob çevir → ses değişir | < 30 ms | < 60 ms | Parametre güncellemesi, tetikleme değil; tolerans daha yüksek. |
| Morph slider sürükleme | < 40 ms | < 80 ms | Sürekli hareket olduğu için algı eşiği yükselir. |

**Uygulama:** WASAPI (exclusive varsa 128 sample @ 48 kHz ≈ 2.7 ms); paylaşımlı modda 256 sample ≈ 5.3 ms. Audio
thread'de **allocation yok**, **kilit yok**, dosya I/O yok. Pahalı işlemler (yeni varyant render'ı) worker thread'de;
UI bunu `[rendering 40 ms]` mikro-rozetiyle gösterir. Ağır işlem sırasında *çalan* voice asla kesilmez.

### 7.2 Tıkırtısız (click-free)

- Her voice başlangıç/bitişinde **3-5 ms** fade.
- Sample/layer değişiminde **5 ms equal-power crossfade**; asla "kes ve değiştir" yok.
- Parametre değişimlerinde gain'e dokunan her şey **10-20 ms** ramp.
- Voice stealing: en eski voice'u **8 ms fade-out** ile sustur, yenisini aynı anda başlat.
- Loop dikişinde minimum **10 ms** crossfade; kullanıcı 0'a çekerse uyarı ver.
- UI sesleri (buton, tick) **-30 dBFS**, 40 ms, yüksek frekanslı; asla sesin üstüne binmez.

### 7.3 Görsel erişilebilirlik

- **Dark UI:** zemin `#0E1116`, panel `#151A21`, yükseltilmiş `#1C232C`, kenar `#2A323D`, metin `#E6EDF3`,
  ikincil `#8B949E`, accent `#4CC2FF`. Kontrast ≥ 4.5:1 (gövde), ≥ 3:1 (büyük metin).
- **Renk körlüğü:** katman renkleri **Okabe-Ito** paletinden: `#0072B2`, `#E69F00`, `#009E73`, `#CC79A7`,
  `#56B4E9`, `#D55E00`, `#F0E442`, `#8B949E`. Renk **tek başına** asla bilgi taşımaz: her katman ayrıca bir
  **şekil rozeti** (● ▲ ■ ◆ ★ ✦) ve **numara** taşır. Renk körü kullanıcı için "renk modu" değil, "şekil modu"
  varsayılan olarak açık.
- **Kontrast uyarısı:** kırmızı metin koyu zeminde kullanılmaz (protanopide siyaha yakın görünür); uyarılar
  `#FF2000` veya `#FFB454` ile.
- Klavye ile her şeye erişim; focus ring 2 px `#4CC2FF`, asla `outline: none` tek başına.
- `prefers-reduced-motion`: playhead dışındaki tüm animasyonlar kapanır.
- Metin ölçekleme %125/%150'de layout kırılmaz (8 px grid, `rem` tabanlı).

---

## 8. Kapsam / MVP kesimi

### 8.1 Özellik tablosu

| Özellik | Karar | Gerekçe |
|---|---|---|
| Audio engine + transport + offline render | **Must** | Her şeyin altında |
| Seed sistemi + deterministik varyant | **Must** | Ürünün kimliği |
| 6 generator (gun, zombie, gore, foley, loop, UI/whoosh) | **Must** | Brief'teki tüm kategoriler |
| Layer editor (stack + gain/pan/pitch/offset) | **Must** | Brief md. 6 |
| Morph slider + parameter map + ease window | **Must** | Brief md. 3 |
| Waveform + playhead + katman şeritleri | **Must** | "Görsel kanıt" |
| Peak/TP/RMS/LUFS metreleri | **Must** | Export doğruluğu |
| Variation grid (20) + keep/discard | **Must** | Brief md. 4 |
| Export: 48 kHz WAV + LUFS + naming + family manifest | **Must** | Brief md. 5 |
| Undo/redo (jest bazlı) | **Must** | Güven |
| Klavye haritası | **Must** | Hız |
| Preset browser (kart + tag) | **Must** | Giriş noktası |
| Spectrogram | **Should** | Güçlü ama waveform yeterli |
| A/B compare (level-matched) | **Should** | Morph kararlarında kritik |
| Distinctness skoru | **Should** | Farklılaştırıcı, maliyeti düşük |
| Loop crossfade editörü | **Should** | Sadece loop kategorisi |
| Drag-to-DAW / Explorer | **Should** | Alışkanlık |
| 90 s capture buffer | **Should** | "Kaybettim" problemini çözer |
| Otomatik tag + UCS metadata | **Should** | Sonradan pahalı |
| XY perform pad + performans kaydı | **Later** | MVP'de tek eksen yeter |
| VST3 plug-in | **Later** | Ayrı bir ürün kararı |
| Node graph / modülasyon matrisi | **Later (belki hiç)** | SFX kullanıcısı istemiyor |
| Cloud library / AI semantic search | **Later** | Yerel kısıt |
| MIDI mapping / OSC | **Later** | Niş |
| Batch project export (tüm oyun) | **Later** | Güçlü v2 satış argümanı |

### 8.2 Gerçekçi inşa sırası (tek geliştirici + AI agent)

| Sprint | İçerik | Gün |
|---|---|---|
| 0 | İskelet, ses cihazı, transport, offline render, WAV writer | 3 |
| 1 | DSP çekirdeği + generator framework + `gun` & `impact` | 6 |
| 2 | Layer stack + timeline + morph map (ease window'lar) | 6 |
| 3 | Seed motoru + variation grid + keep/discard | 5 |
| 4 | Export: LUFS normalizasyon, naming template, manifest | 4 |
| 5 | Preset browser + undo + keybind katmanı | 4 |
| 6 | Kalan generator'lar (zombie, gore, propeller loop, crate, confetti, whoosh, UI) | 8 |
| 7 | Spectrogram, metreler, accessibility pass, onboarding | 4 |
| 8 | Polish: latency tuning, click-free audit, ilk 10 preset ailesi | 5 |
| **Toplam** | | **~45 gün** |

45 gün, "her şey var" için değil, **brief'teki her kategoriyi üreten çalışan bir v1** için. En büyük risk sırası:
(1) generator DSP kalitesi, (2) morph map'in monotonluğu, (3) LUFS/naming doğruluğu. UI riski düşük — çünkü bu
doküman onu sabitliyor.

**Kritik yol notu:** Sprint 1 ve 2'nin *sesi* iyi değilse hiçbir UI kurtarmaz. Sprint 2'nin sonunda 3 preset ailesi
"oyuna konabilir" değilse durup DSP'ye dönülmeli.

---

## 9. Beş somut mockup

### Mockup 1 — Generator View, 1440×900 (varsayılan ekran)

- **Zemin** `#0E1116`. Top bar 0-48 px, `#151A21`, alt kenar 1 px `#2A323D`.
  Sol: "SoundGen" 15 px Inter SemiBold, harf aralığı 0.02em; yanında 4 px `#4CC2FF` dikey çubuk.
  Orta: breadcrumb `Weapon ▸ Pistol ▸ Fire` 12 px `#8B949E`, aktif kısım `#E6EDF3`.
  Sağ: morph slider 280×24 px, track 6 px yuvarlak, dolgu `#4CC2FF`, thumb 16 px daire `#FFFFFF`, ortada 2 px
  `#2A323D` detent. Solda `REAL` 10 px uppercase `#8B949E`, sağda `TOON`. Altında 3 px yükseklikte ghost waveform.
- **Left rail** x = 0-56 px, `#12171E`. Yedi ikon 24×24, 20 px aralıkla; aktif ikon `#4CC2FF`, diğerleri `#6E7781`.
  İkon altında 9 px uppercase label.
- **Orta alan** 56-1120 px. Üstte 40 px başlık: "GENERATOR" 11 px uppercase `#8B949E`, sağda `[Perform]` pill.
  Altında 4 generator kartı, her biri 88 px, aralarında 8 px. Kart: `#151A21`, 1 px `#2A323D`, radius 8.
  Sol kenarda 4 px katman rengi. İçinde: isim 13 px, `⏻ ⚄ ◈` 16 px ikonlar, 5 knob (40 px), sağda 96×48 px scope.
- **Inspector** 1120-1440 px, `#12171E`, sol kenar 1 px. Üstte seçili parametre adı 13 px; altında 200×120 px
  eğri editörü; altında numerik alan (JetBrains Mono 12 px) + birim; en altta "Modulation" listesi.
- **Transport** y = 776-872 px: waveform `#1A2029` zemin, peak `#4CC2FF`, RMS `#4CC2FF` %30; playhead beyaz;
  altta 4 katman şeridi 4 px. Sağda `⟲` loop ve `1.0×` hız.
- **Meter bar** y = 872-900 px, `#0B0E12`, JetBrains Mono 11 px. Her metre 90 px genişlik, aralarında 16 px.

### Mockup 2 — Variation Grid (20 hücre)

- Tam ekran, üstte 56 px araç çubuğu: `Generate 20` (accent buton 120×32), `Spread` slider 160 px,
  `Distinctness 0.71` yeşil pill, sağda `Auto-audition ▸ 1.2s`.
- Grid: 5 sütun × 4 satır, hücre 220×120 px, aralık 12 px, toplam 1160 px; ekranın solunda 24 px boşluk.
- Hücre normal: `#151A21`, 1 px `#2A323D`, radius 8. İçinde: sol üstte seed (JetBrains Mono 10 px `#6E7781`),
  ortada mini waveform 180×40, altta `−14.2 LUFS` ve süre.
- **Keep** edilmiş: sol kenarda 3 px `#3FB950`, sağ üstte 16 px ✓.
- **Discard**: %35 opaklık, üzeri çizili, `X`.
- **Seçili**: 2 px `#4CC2FF` çerçeve + hafif glow.
- **Near-duplicate**: köşede `≈` rozeti `#FFB454`, hover'da "07 ile %97 benzer".
- Alt şerit 72 px: shortlist (keep edilenlerin yatay listesi, 96×48 thumbnail) + `Export family (Ctrl+Shift+E)`.

### Mockup 3 — Morph slider yakın plan (top bar, 280×24)

- Track: 280×6 px, radius 3, `#1C232C`. Sol yarı `#0072B2`→`#4CC2FF` gradyan (real = soğuk), sağ yarı
  `#FFB454`→`#E69F00` (toon = sıcak). Merkezde 2×10 px `#0E1116` detent çentiği.
- Thumb: 16 px beyaz daire, 2 px `#0E1116` halka, hover'da 18 px.
- Altında 5 tik (0, .25, .5, .75, 1), 4 px yüksek, `#2A323D`.
- Üstünde sağa hizalı değer: `+42` (JetBrains Mono 11 px), birim yok — yüzde değil, *stil puanı*.
- Ghost waveform: track'in arkasında 24 px yükseklikte, real silueti `#2A323D`, toon silueti `#3A4654`;
  mevcut değerde iki siluet arasında interpolasyon.
- Sağda küçük `◈` lock (morph'un kendisini A/B arasında sabitlemek için) ve `A|B` iki harf.

### Mockup 4 — Export sayfası

- 720×560 px modal değil, **tam ekran sayfa** (export bir akış). Üç sütun 1:1:1, 24 px gutter.
- **Sol — Naming:** canlı önizleme kutusu `#0B0E12`, JetBrains Mono 13 px:
  `sfx_wpn_pistol_fire_03_s00871.wav`. Altında token'lar chip olarak: `{cat}`, `{name}`, `{index}`, `{seed}`,
  `{lufs}`. UCS modu toggle'ı: `GUNAuto_PistolFire_03_SNDGN.wav`.
- **Orta — Audio:** Sample rate `48000` (kilitli, gri), channels `Mono/Stereo` segmented control,
  bit depth `24`, normalize `Short-term LUFS` + hedef `-14.0` + true-peak ceiling `-1.0 dBTP`.
  Altta mini loudness strip: sesin S-LUFS eğrisi + kırmızı hedef çizgisi.
- **Sağ — Delivery:** klasör seçici, `Write manifest (.family.json)` checkbox, `Copy to Unity Resources` checkbox.
- Alt sağ: `Export` (accent, 140×36) + `Export & Reveal`. Alt sol: `3 variants, 14.2 MB, est. 0.8 s`.

### Mockup 5 — Preset Browser kartları

- Sol panel 240 px: kategori ağacı, 13 px, satır 28 px; seçili satır `#1C232C` + 3 px accent.
- Kart grid: 3 sütun × 240 px, gutter 16 px, başlangıç 280 px. Kart 240×132 px, `#151A21`, radius 10.
- Kart içinde: üstte 240×48 px waveform (katman renkleriyle), altında isim 13 px `#E6EDF3`,
  alt satırda `0.8s · −14.2 LUFS` 11 px `#8B949E`, sağ altta 3 tag pill (8 px radius, `#1C232C`, 10 px).
- Hover: kart 1 px `#4CC2FF`, waveform canlı oynar (preview), sağ üstte `▸` butonu.
- Boş durum: "No presets in this category — press `Ctrl+N` to generate from a template." 13 px `#8B949E`.

---

## 10. Açılan kaynaklar

**Gerçek ekran görüntüleri incelendi:** Enrage (Gearnews), Reformer Pro (MusicRadar), Weaponiser (Krotos resmî UI),
Phase Plant (Kilohearts).

1. MusicRadar — *Krotos Reformer Pro review* — https://www.musicradar.com/reviews/krotos-audio-reformer-pro
2. MusicTech — *Krotos Reformer Pro Review* — https://musictech.com/reviews/krotos-reformer-pro-review/
3. MusicTech — *Krotos Weaponiser Review* — https://musictech.com/reviews/krotos-audio-weaponiser-review/
4. The Sound Architect — *Weaponiser by Krotos Review* — https://www.thesoundarchitect.co.uk/weaponiser-by-krotos-review/
5. Sound on Sound — *Krotos Studio* — https://www.soundonsound.com/reviews/krotos-studio
6. Krotos — *Getting to grips with Krotos Studio* — https://www.krotosaudio.com/getting-to-grips-with-krotos-studio/
7. Bedroom Producers Blog — *EnRage Review* — https://bedroomproducersblog.com/2021/05/18/enrage-review/
8. Gearnews — *Enrage first look* — https://www.gearnews.com/first-look-enrage-is-a-beast-of-a-multi-fx-plug-in-that-needs-some-taming/
9. Synth and Software — *Enrage review* — https://synthandsoftware.com/2021/06/boom-library-enrage-modular-multi-effects-plug-in-the-proverbial-desert-island-processor/
10. Boom Library — *Turbine Manual v1.1.4* — https://www.boomlibrary.com/manuals/00_Turbine_Manual_V1.1.4.pdf
11. 344 Audio — *Turbine review* — https://www.344audio.com/post/product-review-boom-library-turbine
12. Post Perspective — *Review: Soundly* — https://postperspective.com/review-soundly-an-essential-tool-for-sound-designers/
13. Soundly — https://getsoundly.com/
14. Kilohearts — *Phase Plant docs* — https://kilohearts.com/docs/phase_plant
15. Synth and Software — *Phase Plant 2.1 review* — https://synthandsoftware.com/2023/09/kilohearts-phase-plant-2-1-the-synth-and-software-review/
16. MusicRadar — *Kilohearts Phase Plant review* — https://www.musicradar.com/reviews/kilohearts-phase-plant
17. Vital — https://vital.audio/ ; *Modulation docs* — https://davidmvogel.com/docs/Vital/UserGuide/Modulation
18. RouteNote — *Vital review* — https://routenote.com/blog/vital-matt-tytel-review/
19. MusicTech — *Arturia Pigments 7 review* — https://musictech.com/reviews/software-instruments/arturia-pigments-7-review/
20. MusicRadar — *Arturia Pigments review* — https://www.musicradar.com/music-tech/synths/arturia-pigments-review
21. MusicTech — *UVI Falcon 3 review* — https://musictech.com/reviews/software-instruments/uvi-falcon-3-review/
22. MusicRadar — *UVI Falcon 3 review* — https://www.musicradar.com/reviews/uvi-falcon-3-review
23. Syntorial — *Serum review* — https://www.syntorial.com/highlights/xfer-serum/
24. Audiotent — *Adding randomness to synth presets in Serum* — https://www.audiotent.com/blogs/production-tips/adding-randomness-to-synth-presets-in-xfer-serum
25. VI-Control — *Any synth with Random button?* — https://vi-control.net/community/threads/any-synth-with-random-button-to-create-sounds.100533/
26. A Sound Effect — *REAPER for Game Audio: Getting Started & Rendering* — https://www.asoundeffect.com/reaper-for-game-audio-getting-started-rendering/
27. Ableton — *Working with the Browser* — https://www.ableton.com/en/manual/working-with-the-browser/
28. Ableton — *Live Keyboard Shortcuts* — https://www.ableton.com/en/manual/live-keyboard-shortcuts/
29. Audiokinetic — *Creating Random Containers* — https://www.audiokinetic.com/en/library/edge/?id=creating_random_container
30. FMOD Q&A — *Random playlist guarantee no repeats* — https://qa.fmod.com/t/random-playlist-guarantee-no-repeats/11852
31. EBU — *R 128 s1: Loudness parameters for short-form content* — https://tech.ebu.ch/files/live/sites/tech/files/shared/r/r128s1.pdf
32. Anso Audio — *Loudness and metering in game audio* — https://ansoaudio.com/2016/07/27/loudness-and-metering-in-game-audio/
33. Lester & Boley — *The Effects of Latency on Live Sound Monitoring* (AES) — https://cdn.abicart.com/shop/ws42/76342/art85/220769785-35ade3-AES_Latency.pdf
34. *Measuring the Just Noticeable Difference for Audio Latency* (2024) — https://doi.org/10.1145/3678299.3678331
35. Universal Category System — https://universalcategorysystem.com/ ; JamSoft özeti — https://www.jamsoft.co.uk/samplesort/support/universal-category-system/
36. Okabe & Ito — *Color Universal Design* — https://jfly.uni-koeln.de/color/ ; hex listesi — https://siegal.bio.nyu.edu/color-palette/
37. Tone.js — https://tonejs.github.io/ ; *Context/latencyHint* — https://tonejs.github.io/docs/r13/Context ; *Performance* — https://github.com/Tonejs/Tone.js/wiki/Performance
38. AudioKit — https://www.audiokit.io/ (platform: yalnızca Apple)
39. Wikipedia — *Comparison of audio synthesis environments* — https://en.wikipedia.org/wiki/Comparison_of_audio_synthesis_environments
40. Andy Farnell — *Procedural Audio*, AES UK lecture — https://www.youtube.com/watch?v=sp83-Pq7TyQ

---

## 15 satırlık öneri özeti

1. Ürünün çekirdeği **"tek ses" değil "aile"**: her preset bir `seedRoot` + varyant kümesi olarak yaşamalı.
2. Ana ekran **Generator**, çünkü kullanıcı "hangi parçalar" sorusunu "hangi knob" sorusundan önce soruyor.
3. **Transport hiç kaybolmamalı** — ses her ekranda çalmaya devam etmeli; bu, DAW'lardan alınacak en değerli alışkanlık.
4. **Realism↔Cartoon tek bipolar slider** olacak; XY pad sadece Generator'da, ikinci eksen `Size`.
5. Morph **crossfade değil parameter morph**; tek render, `ease window`'lu parametre haritası, ayrık eşikler 30 ms fade ile.
6. Katman başına `morphBias` + parametre başına **lock** — tek eksen kararının sakıncasını bunlar sıfırlıyor.
7. Her katman **Okabe-Ito rengi + şekil rozeti + numara**; renk asla tek başına bilgi taşımayacak.
8. **`R` rastgeleleştir, `L` kilitle, `Space` çal, `K` tut** — dört tuş ürünün omurgası.
9. 20 varyantı **60 saniyede** bitirmek ölçülebilir bir hedef; auto-audition + keep/discard bunun için var.
10. **Distinctness skoru** ekle: 20 benzer dosya, 6 farklı sesten değersizdir.
11. Export **tek sayfa**, canlı isim önizlemesi, `*.family.json` manifest ve `avoidRepeatLast: 3`.
12. Kısa SFX'te **integrated LUFS anlamsız**; short-term/momentary + true-peak −1 dBTP kullan.
13. Gecikme hedefi **< 10 ms** (WASAPI 128-256 sample); audio thread'de allocation ve kilit yok.
14. MVP **~45 gün**; en büyük risk UI değil, generator DSP kalitesi ve morph monotonluğu.
15. Yapılmayacaklar: node graph, VST3, bulut, performans kaydı — v1'de bunlar odağı dağıtır.

**Dosya yolu:** `C:\Users\kyusu\dev\soundGen\docs\research\02-arac-ux.md`
