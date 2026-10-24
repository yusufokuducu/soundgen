# 04 — Entegrasyon ve Teslim Sözleşmesi (Audio Delivery Contract)

**Durum:** v1.0 — normatif taslak
**Kapsam:** SoundGen'in ürettiği her ses varlığının dosya formatı, loudness standardı, isimlendirme/metadata, Unity entegrasyonu, gelecekteki FMOD/Wwise geçişi, oyun içi varyasyon stratejisi ve kabul (QA) kriterleri.
**Hedef tüketici:** Walkin (Unity 6, DOTS/ECS, mobile + PC), ileride FMOD Studio / Wwise.
**Anahtar kelimeler:** `MUST` = ZORUNLU, `SHOULD` = ÖNERİLEN, `MAY` = İSTEĞE BAĞLI. Bu kelimeler RFC 2119 anlamında kullanılmıştır; validator bu üç seviyeyi ayrı ayrı raporlar.

Bu doküman bir "fikir yazısı" değil, **kopyalanabilir sözleşme metnidir**. Buradaki kurallar SoundGen export pipeline'ının kabul kapısıdır (acceptance gate): kurala uymayan asset oyuna girmez, `qa.passed = false` ile karantinaya düşer.

---

## 0. Temel ilkeler (bütün bölümler için geçerli)

1. **Tek doğruluk kaynağı (single source of truth) ses dosyası değil, sidecar JSON'dur.** WAV yalnızca örnekleri taşır; anlam (kategori, loudness, loop noktaları, katman rolleri) JSON'dadır. WAV ile JSON çelişirse build **FAIL** eder.
2. **Generator asla "mix" yapmaz, "gain stage" yapar.** SoundGen'in işi her asset'i kategori hedefine getirmek; nihai denge oyunun AudioMixer'ında kurulur. Bu ayrım korunmazsa hem yeniden dengeleme hem de middleware geçişi imkânsızlaşır.
3. **Ölçülebilir olmayan kural yoktur.** Bu dokümandaki her sayısal eşik `qa.checks` altında otomatik ölçülür ve sidecar'a yazılır.
4. **Determinizm.** Aynı `seed` + aynı `preset` + aynı generator sürümü aynı dosyayı üretmelidir (`provenance.sha256` ile doğrulanır). Dither nedeniyle 16-bit çıktıda son bit oynayabilir; 24-bit master'da birebir eşleşme ZORUNLU'dur.
5. **Platform bağımsız master, platforma özgü türev.** Master her zaman 48 kHz WAV'dır; Unity/FMOD/Wwise türevleri aynı master'dan üretilir, elle düzenlenmez.

---

## 1. Dosya formatı sözleşmesi

### 1.1 Master ve teslim formatı

| Katman | Format | Sample rate | Bit depth | Kanal | Not |
|---|---|---|---|---|---|
| Internal DSP / render | float32 (WAV RF64 gerekirse) | 48 000 | 32-bit float | kaynak kanal sayısı | Headroom sınırsız, clipping yok |
| **Master (arşiv)** | WAV PCM (RIFF) | **48 000** | **24-bit** | politikaya göre | Dither YOK |
| Unity teslim (PCM yolu) | WAV PCM | 48 000 | 16-bit | politikaya göre | TPDF dither, −1 dBTP |
| Unity teslim (sıkıştırılmış yol) | WAV PCM (kaynak) → engine codec | 48 000 | 16-bit | politikaya göre | Unity içinde Vorbis/ADPCM'e çevrilir |
| FMOD / Wwise | WAV PCM | 48 000 | 24-bit | politikaya göre | Dither YOK, normalize edilmiş |
| Raw / araştırma | WAV 32-bit float | 48 000 | 32-bit float | ham | Gain uygulanmamış, sadece trim |

**Kurallar:**

- Sample rate **MUST** 48 000 Hz. 44.1 kHz teslim edilmez; SRC artefaktı ve loop uzunluğu hatası üretir. (Oyun endüstrisi standardı 48 kHz; Critical Listening Lab ve Unity dokümantasyonu da bunu varsayar.)
- 96 kHz yalnızca **master render** aşamasında, 6 semitonun üzerinde pitch-down yapılacak asset'lerde **MAY** kullanılır; teslim yine 48 kHz'dir (yüksek kaliteli SRC ile).
- Bit depth 24 (master) / 16 (teslim). 16-bit'e inerken **TPDF dither MUST**; 24-bit'e asla dither uygulanmaz.
- Dosya adı uzantısı `.wav`, container **RIFF/WAVE**, codec **PCM**. `.ogg`, `.mp3`, `.aiff` teslim edilmez (Unity bunları import edebilir ama generator'ın çıktısı değildir).
- Ek chunk'lar: `LIST/INFO` (opsiyonel, `INAM`, `ICMT`), `smpl` (loop varsa **SHOULD**), `bext` (arşiv master'da **MAY**). `smpl` dışındaki özel chunk'lar Unity tarafından yok sayılır.

### 1.2 Kanal politikası (kategori bazlı)

3D konumlandırılan her ses **mono** olmalıdır. Stereo yalnızca gerçekten genişlik taşıyan içerik içindir. Bu kural hem bellek hem de spatialization doğruluğu içindir: stereo bir 3D source Unity'de "spread" olarak yorumlanır, sahne netliğini bozar.

| Kategori | Kanal | Gerekçe |
|---|---|---|
| `weapon.*` (ateş, mekanizma, kovan, susturucu) | **mono** | 3D, tek noktadan gelir; stereo versiyon sadece 1st-person "body" katmanı için `MAY` |
| `zombie.*` (vocal, horde) | **mono** | Her instance ayrı konumlanır; horde bed'i ayrı `AMB_` asset'i olarak stereo |
| `gore.*` / `impact.*` | **mono** | Konumsal |
| `foley.*` (adım, kumaş, silah takılması) | **mono** | Konumsal, çok sayıda eşzamanlı |
| `ui.*` | **mono** | 2D, merkez; stereo gereksiz bellek |
| `whoosh.*`, `transition.*` | **mono** (stereo `MAY`, sadece 2D kullanılacaksa) | Genelde 3D değil ama merkezde |
| `ambience.*` | **stereo** | Yatak (bed), genişlik taşır |
| `loop.propeller/drone/engine` | **mono** (3D) / **stereo** (cockpit, 2D) | Kullanım yerine göre; iki versiyon `MAY` |
| `celebration.confetti` | **stereo** | Kutlama, genişlik algısı hedeflenir |
| `music.*` / stinger | **stereo** | 2D |

**Stereo teslim ediliyorsa mono uyumluluk ZORUNLU'dur:** L+R toplamı, stereo seviyenin −3 dB'sinden fazla düşmemeli; 100 Hz–8 kHz aralığında hiçbir 1/3-oktav bandında 6 dB'den fazla iptal olmamalı (korelasyon katsayısı > 0.2).

Generator hiçbir zaman 2 kanaldan fazlasını üretmez. Force To Mono'yu "düzeltme" olarak kullanmak yasaktır (bkz. §4.2).

### 1.3 Headroom ve peak politikası

| Asset tipi | Sample peak | True peak | Neden |
|---|---|---|---|
| Tek atım (one-shot) SFX | ≤ −1.0 dBFS | **≤ −1.0 dBTP** | Codec overshoot payı |
| Loop / ambience / bed | ≤ −3.0 dBFS | **≤ −3.0 dBTP** | Üst üste binme (summing) payı |
| Katman stem'i (`LY`) | ≤ −6.0 dBFS | **≤ −6.0 dBTP** | Stem'ler toplanınca parent'a ulaşır |
| UI / music stinger | ≤ −1.0 dBFS | ≤ −1.0 dBTP | 2D, tek kanal yol |

Gerekçe: Unity'de pitch randomization, eşzamanlı varyasyonlar ve layering tepe seviyeyi 3–6 dB yukarı taşır. −1 dBTP, lossy codec'in (Vorbis) yeniden örnekleme sırasında ürettiği intersample overshoot için standart güvenlik payıdır. Nihai mix bus'ta **limiter yoktur**; sadece master'da −1 dBTP ceiling uygulanır.

### 1.4 Tail (kuyruk) yönetimi

- Doğal decay −60 dBFS'in altına inene kadar korunur. Decay **−40 dBFS'in üzerinde kesilemez** (MUST).
- Kategori bazlı maksimum tail: `weapon.gunshot` 2.5 s, `gore` 1.5 s, `foley` 0.8 s, `ui` 0.3 s, `crate.open` 3.0 s, `celebration` 4.0 s. Bu sürelerin ötesi reverb/uzaklık katmanıdır, ayrı asset'tir.
- Tail'in sonuna **5–15 ms fade-out** uygulanır (hard cut click üretir). `−60 dBFS` eşiğinden sonra kalan < 2 ms kuyruk fade'e dahil edilir.
- Ambient/loop asset'lerinde tail dosyanın **içine** sarılır (circular convolution); loop dışında tail bırakılmaz.

### 1.5 Loop sözleşmesi (pervane, drone, ambience, motor)

Loop, bu projede en kolay bozulan teslim kalemidir; bu yüzden **"tek dosya = tek tam periyot"** kuralı benimsenir.

**Zorunlu kurallar:**

1. `loop.mode = "whole_file"` — dosya baştan sona çalar ve `AudioSource.loop = true` ile döner. `smpl` chunk'ı **SHOULD** yazılır (Wwise/Unity uyumu için) ama oyun mantığı ona **bağımlı olmaz**.
2. Dosya **intro içermez**. Intro gerekiyorsa `..._IN.wav` + `..._LP.wav` olarak iki asset üretilir.
3. Loop uzunluğu: pervane/drone **2.0–8.0 s**, ambience **20–60 s**, motor **1.0–4.0 s**.
4. **Tonalli loop'larda** (pervane, motor) döngü, dönüş periyodunun tam katı olmalıdır. Blade-pass frekansı `f_bp` ise `loop_length_sec = k / f_bp` (k tam sayı). Aksi hâlde duyulabilir "vuruş" (beating) oluşur.
5. **Broadband loop'larda** (rüzgâr, kalabalık) eşit-güçlü (equal-power) crossfade kullanılır: son `X` ms ile ilk `X` ms çaprazlanır, ardından dosya `X` ms kısaltılır. `X` = 20–50 ms (gürültü), 100–250 ms (tonal ağırlıklı), `loop.crossfade_ms` alanına yazılır.
6. **Süreklilik testi geçmeden asset kabul edilmez** (bkz. §8): sınır sıçraması, 10 ms RMS farkı, 1/3-oktav spektral akış sıçraması.
7. Loop içinde **sessizlik boşluğu olamaz**: hiçbir 200 ms pencere −60 dBFS altında kalamaz (dropout).
8. Loop'un stereo versiyonunda L/R korelasyonu sabit kalmalı; loop noktasında genişlik sıçraması yasak.

**Unity gerçeği:** Unity, WAV `smpl` chunk'ını **belgelemez ama okur** (yalnızca ilk loop bölgesi, ve yalnızca PCM / Decompress On Load yollarında güvenilir; Vorbis yeniden kodlaması loop noktalarını kaydırır veya siler). Bu yüzden sözleşme "whole_file"dur: metadata'ya güvenmeyen, her koşulda çalışan yol. `smpl` yine de yazılır çünkü Wwise ve gelecekteki araçlar onu kullanır.

### 1.6 Trim ve fade kuralları

| Kural | Eşik | Uygulama |
|---|---|---|
| Başlangıç sessizliği | ≤ **2 ms** (32 sample) ve ≤ −60 dBFS | Transient sample 0'da başlar |
| Fade-in | **0 ms** (transient SFX) / 0.5–1 ms (DC veya click varsa) | Attack asla yumuşatılmaz |
| Bitiş fade-out | 5–15 ms (perküsif), 20–50 ms (whoosh/geçiş), 0 ms (loop) | −60 dBFS sonrası |
| Sondaki sessizlik | ≤ 250 ms (one-shot), 0 ms (loop) | Ölçüm ve bellek için |
| DC offset | \|mean\| ≤ 0.001 FS (−60 dBFS) | High-pass 20 Hz, linear phase, sadece gerekliyse |

**Neden başlangıçta fade-in yok:** Bir silah sesinin ilk 2 ms'i tüm "punch" bilgisini taşır. Fade-in uygulamak transient'i öldürür ve sesi "yumuşak/oyuncak" yapar. Fade-in yalnızca dosya DC ile başlıyorsa veya ilk sample'da click varsa uygulanır ve 1 ms'i geçmez.

---

## 2. Loudness standardı

### 2.1 Ölçüm temeli: ITU-R BS.1770 / EBU R128

Ölçüm **ITU-R BS.1770-5** algoritmasına dayanır: K-weighting filtresi (yüksek geçiren + yüksek frekans rafı), kanal ağırlıklı enerji toplamı, 400 ms bloklar (%75 örtüşmeli), ve true-peak için en az 4× oversampling.

- **Absolute gate:** −70 LUFS altındaki bloklar hesaptan çıkar.
- **Relative gate:** absolute-gated ortalamanın 10 LU altındaki bloklar da çıkar.
- **Momentary (M):** 400 ms pencere, gate yok.
- **Short-term (S):** 3 s pencere, gate yok.
- **Integrated (I):** tüm program, iki kapılı (gated) ortalama.
- **LRA:** gated loudness range, 10.–95. persentil.

EBU R128 yayın hedefi: **−23 LUFS ± 0.5 LU**, true peak **≤ −1 dBTP**. ABD'de ATSC A/85: −24 LKFS ± 2, −2 dBTP. Oyun tarafında Sony ASWG-R001: konsol **−23 LUFS ± 2**, taşınabilir **−18 LUFS**, **−1 dBTP** tavan.

### 2.2 Neden "integrated LUFS" tek başına yanlış hedeftir

Bir SFX'in süresi çoğu zaman **400 ms'den kısadır**. Bu durumda:

1. **Integrated ölçüm tanımsızlaşır.** BS.1770'in en küçük bloğu 400 ms'dir; 90 ms'lik bir UI click için "integrated" değeri, ölçüm dosyasının etrafına eklenen sessizliğe bağlı olarak 10 LU'ya kadar kayar. Aynı ses, farklı sessizlik dolgusuyla farklı LUFS verir — yani sayı asset'in değil, ölçüm dosyasının özelliğidir.
2. **Gating kısa sesi yer.** Relative gate, absolute-gated ortalamanın 10 LU altını atar. Kısa bir transient'in decay kuyruğu ve varsa başlangıç sessizliği bu kapıya takılır; geriye sadece en yüksek blok kalır. Sonuç: ölçüm tek bir 400 ms bloğuna indirgenir, yani "momentary" zaten.
3. **Transient bilgisi kaybolur.** İki farklı gunshot aynı integrated değerine sahip olabilir ama biri 3 dB daha "sert" duyulur; fark peak-to-loudness ratio (PLR) ve spektral içeriktedir. Integrated bunu görmez.

**Bu yüzden sözleşme `integrated` yerine `REL` (Reference Event Loudness) tanımlar:**

> **REL** = BS.1770 K-weighted, **gatesiz** ortalama enerji, **dosyanın tamamı** üzerinden, LUFS olarak. Durağan (steady) içerikte REL ≈ M-max (±0.5 dB); dinamik içerikte REL, M-max'ın altında kalır ve aradaki fark PLR'ın bir parçasıdır. Süre < 400 ms ise ölçüm penceresi zaten dosya uzunluğudur, dolayısıyla REL tek anlamlı metriktir.

Her asset için üç sayı zorunludur: **REL**, **M-max (veya S-max)**, **true peak**. Dördüncü olarak **PLR = true_peak − REL** raporlanır ve kategori aralığında olmalıdır (aşırı limitlenmiş ses = düşük PLR = "ezilmiş" ses).

Loop ve ambience gibi sürekli içerikte `integrated` **anlamlıdır** ve tek hedef olarak kullanılabilir (≥ 10 s ölçüm üzerinden). Müzik/stinger'da `integrated` kullanılır (AES TD1008 müzik dağıtım hedefi −16 LUFS).

### 2.3 Kategori bazlı hedefler

`measure` alanı hangi metriğin hedefe vurulduğunu söyler. Tolerans **±1.5 LU** (BLOCKER dışı, WARN), hedefe 1.0 LU'dan fazla sapma build uyarısı üretir.

| `asset_id` kategorisi | `measure` | Hedef (LUFS) | TP ceiling | PLR beklentisi | Süre |
|---|---|---|---|---|---|
| `weapon.gunshot.player` | momentary_max | **−14** | −1 dBTP | 14–24 dB | 0.4–1.5 s |
| `weapon.gunshot.npc` | momentary_max | **−18** | −1 dBTP | 14–24 dB | 0.4–1.5 s |
| `weapon.suppressed` | momentary_max | **−20** | −1 dBTP | 12–20 dB | 0.2–0.8 s |
| `weapon.mechanism` (reload, bolt, shell) | momentary_max | **−22** | −1 dBTP | 10–18 dB | 0.1–0.6 s |
| `zombie.vocal.single` | momentary_max | **−18** | −1 dBTP | 10–18 dB | 0.5–3.0 s |
| `zombie.vocal.horde` (bed) | short_term_max | **−24** | −1 dBTP | 6–14 dB | loop |
| `gore.*` / `impact.*` | momentary_max | **−16** | −1 dBTP | 12–22 dB | 0.2–1.2 s |
| `foley.footstep` | momentary_max | **−26** | −1 dBTP | 10–20 dB | 0.08–0.4 s |
| `foley.cloth` | momentary_max | **−28** | −1 dBTP | 8–18 dB | 0.1–0.5 s |
| `ambience.*` (bed) | short_term_max | **−28** | −3 dBTP | 6–16 dB | 20–60 s loop |
| `loop.propeller` / `loop.engine` | short_term_max | **−24** | −3 dBTP | 4–12 dB | 2–8 s loop |
| `ui.click` | momentary_max | **−20** | −1 dBTP | 10–20 dB | 0.04–0.15 s |
| `ui.reward` / `level_up` | integrated | **−16** | −1 dBTP | 8–18 dB | 0.8–2.5 s |
| `whoosh.*` / `transition.*` | momentary_max | **−18** | −1 dBTP | 10–20 dB | 0.3–1.2 s |
| `crate.open` | momentary_max | **−17** | −1 dBTP | 12–22 dB | 0.8–3.0 s |
| `celebration.confetti` | momentary_max | **−15** | −1 dBTP | 10–20 dB | 1.0–4.0 s |
| `music.stinger` | integrated | **−16** | −1 dBTP | 8–20 dB | 2–8 s |

**Gerekçe ve referanslar:**

- Oyun içi SFX için −18…−23 LUFS aralığı, mix'e headroom bırakmak için yaygın kabul görmüş teslim aralığıdır; müzik −14…−16 LUFS'a normalize edilir (Critical Listening Lab oyun spesifikasyonu; AES TD1008 müzik dağıtım hedefi −16 LUFS).
- Oyuncu silahı gibi "hikâye anlatan" sesler kategorik olarak daha yüksek tutulur (−14); foley ve UI daha düşük. Bu, **fader'ları 0 dB'de bırakıp önemi loudness hedefine gömmek** demektir; böylece mix yeniden kurulabilir kalır (kategori trim'leri AudioMixer'da, snapshot'larda).
- Tüm değerler **gain-only normalizasyon** ile uygulanır. SFX'e limiter uygulamak transient'i ve PLR'ı yok eder; yasaktır.

---

## 3. İsimlendirme ve metadata sözleşmesi

### 3.1 Dosya adı şeması

```
<PREFIX>_<DOMAIN>_<SUBJECT>_<ACTION>_<VARIANT>[_<FLAGS>].wav
```

- `PREFIX` ∈ `SFX | AMB | UI | VOX | MUS` — doğrudan AudioMixer grubuna eşlenir.
- `DOMAIN` ∈ `WEAPON | ZOMBIE | GORE | FOLEY | CRATE | CELEB | UI | WHOOSH | AMB | MUSIC`.
- `SUBJECT` somut nesne: `PISTOL`, `RIFLE`, `SHOTGUN`, `HEAD`, `FLESH`, `BONE`, `WOOD`, `METAL`, `PROPELLER`, `WIND`.
- `ACTION` fiil-isim: `FIRE`, `RELOAD`, `SHELLDROP`, `BITE`, `SCREAM`, `GROAN`, `DEATH`, `STEP`, `OPEN`, `BURST`, `CLICK`, `REWARD`.
- `VARIANT` = `V01`…`V99` (iki hane, sıfır dolgulu). Aynı `family` içinde tekil.
- `FLAGS` (opsiyonel, sıralı, `_` ile): `RL` round-robin/loop, `LP` loop, `ST` stereo, `NR` near, `FR` far, `LY` layer stem, `IN` intro, `2D` non-positional.

**Örnekler:**

```
SFX_WEAPON_PISTOL_FIRE_V01_RL.wav
SFX_WEAPON_PISTOL_FIRE_V01_LY_TRANSIENT.wav
SFX_ZOMBIE_HEAD_BITE_V03.wav
VOX_ZOMBIE_MALE_SCREAM_V07_FR.wav
AMB_SWAMP_WIND_LOOP_V01_LP_ST.wav
SFX_PROPELLER_DRONE_LOOP_V01_LP.wav
UI_MENU_BUTTON_CLICK_V01.wav
MUS_LEVELUP_STINGER_V01_ST.wav
```

**Yasaklar:** Türkçe karakter, boşluk, tire, parantez, 3 haneli varyant (`V001` değil `V01`), 40 karakterden uzun `SUBJECT`, tarih/sürüm gibi değişken bilgi (bu bilgi JSON'a gider, ada değil).

**Regex (validator için):**

```regex
^(SFX|AMB|UI|VOX|MUS)_(WEAPON|ZOMBIE|GORE|FOLEY|CRATE|CELEB|UI|WHOOSH|AMB|MUSIC)_[A-Z0-9]{2,20}_[A-Z0-9]{2,20}_V\d{2}(_(RL|LP|ST|NR|FR|LY|IN|2D|TRANSIENT|BODY|TAIL|MECH|DEBRIS|SUB|AIR|TONAL|NOISE))*\.wav$
```

### 3.2 Klasör yapısı (Unity ile birebir)

```
Assets/Audio/
  SFX/Weapon/Pistol/Fire/SFX_WEAPON_PISTOL_FIRE_V01_RL.wav
  SFX/Weapon/Pistol/Fire/SFX_WEAPON_PISTOL_FIRE_V01_RL.wav.sfx.json
  AMB/Swamp/AMB_SWAMP_WIND_LOOP_V01_LP_ST.wav
  UI/Menu/UI_MENU_BUTTON_CLICK_V01.wav
  MUS/Stingers/MUS_LEVELUP_STINGER_V01_ST.wav
```

**Kritik:** Sidecar dosya adı **`<dosya>.wav.sfx.json`** olmalıdır. Sadece `.json` kullanmak Unity'nin `X.wav.meta` dosyasıyla çakışmaz ama `X.json` + `X.wav.meta` iki ayrı meta dosyası gibi görünür ve taşıma sırasında kaybolur; `X.wav.sfx.json` WAV ile birlikte hareket eder ve Unity onu `TextAsset` olarak import eder (import edilmemesi için klasörde `.asmdef` dışı tutulur veya `AudioPostprocessor` bunu `DefaultAsset` olarak işaretler).

### 3.3 `event_id` / isim uzayı

Dosya adı **fiziksel**, `event_id` **mantıksal** anahtardır. İkisi ayrıdır; oyun kodu yalnızca `event_id` bilir.

```
weapon.pistol.fire
weapon.pistol.fire.layer.transient
zombie.male.scream
amb.swamp.wind
ui.menu.click
```

Middleware karşılıkları:

| Sistem | Karşılık |
|---|---|
| Unity Addressables | `address = "sfx/weapon/pistol/fire/v01"` |
| FMOD | `event:/SFX/Weapon/Pistol/Fire` |
| Wwise | `Play_SFX_Weapon_Pistol_Fire` |

### 3.4 Sidecar JSON şeması (tam)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://soundgen.local/schema/sfx-asset-1.0.schema.json",
  "title": "SoundGen SFX Descriptor",
  "description": "SoundGen'in ürettiği her ses varlığı için tek doğruluk kaynağı.",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "schema", "asset_id", "file", "category", "tags", "seed", "style",
    "audio", "loudness", "loop", "trim", "layers", "variation", "mix",
    "engine", "qa", "provenance"
  ],
  "properties": {
    "schema": { "const": "soundgen.sfx/1.0" },
    "asset_id": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9]*(\\.[a-z0-9]+)+$",
      "description": "Mantıksal anahtar, örn. weapon.pistol.fire"
    },
    "file": {
      "type": "string",
      "pattern": "^[A-Z0-9_]+\\.wav$"
    },
    "category": {
      "enum": ["weapon", "zombie", "gore", "foley", "ambience", "loop",
               "crate", "celebration", "ui", "whoosh", "music"]
    },
    "subcategory": { "type": "string", "pattern": "^[a-z0-9._-]+$" },
    "tags": {
      "type": "array",
      "uniqueItems": true,
      "maxItems": 32,
      "items": { "type": "string", "pattern": "^[a-z0-9][a-z0-9:_-]*$" }
    },
    "seed": { "type": "integer", "minimum": 0 },
    "style": {
      "type": "number", "minimum": 0, "maximum": 1,
      "description": "0 = gerçekçi, 1 = animasyon/abartılı"
    },
    "audio": {
      "type": "object",
      "additionalProperties": false,
      "required": ["sample_rate", "bit_depth", "channels", "format",
                   "duration_sec", "samples"],
      "properties": {
        "sample_rate": { "const": 48000 },
        "bit_depth": { "enum": [16, 24, 32] },
        "channels": { "enum": [1, 2] },
        "format": { "const": "wav_pcm" },
        "duration_sec": { "type": "number", "exclusiveMinimum": 0 },
        "samples": { "type": "integer", "minimum": 1 }
      }
    },
    "loudness": {
      "type": "object",
      "additionalProperties": false,
      "required": ["measure", "target_lufs", "rel_lufs",
                   "momentary_max_lufs", "short_term_max_lufs",
                   "true_peak_dbtp", "gain_applied_db"],
      "properties": {
        "measure": {
          "enum": ["momentary_max", "short_term_max", "integrated"]
        },
        "target_lufs": { "type": "number" },
        "rel_lufs": { "type": "number" },
        "momentary_max_lufs": { "type": "number" },
        "short_term_max_lufs": { "type": "number" },
        "integrated_lufs": { "type": "number" },
        "loudness_range_lu": { "type": "number", "minimum": 0 },
        "true_peak_dbtp": { "type": "number", "maximum": -1.0 },
        "peak_dbfs": { "type": "number", "maximum": -1.0 },
        "plr_db": { "type": "number", "minimum": 0 },
        "gain_applied_db": { "type": "number" },
        "meter": { "type": "string" }
      }
    },
    "loop": {
      "type": "object",
      "additionalProperties": false,
      "required": ["is_loop", "mode", "seamless_verified"],
      "properties": {
        "is_loop": { "type": "boolean" },
        "mode": { "enum": ["none", "whole_file", "smpl"] },
        "start_sample": { "type": "integer", "minimum": 0 },
        "end_sample": { "type": "integer", "minimum": 1 },
        "start_sec": { "type": "number", "minimum": 0 },
        "end_sec": { "type": "number", "minimum": 0 },
        "crossfade_ms": { "type": "number", "minimum": 0 },
        "periods": { "type": "integer", "minimum": 1 },
        "blade_pass_hz": { "type": "number", "minimum": 0 },
        "seamless_verified": { "type": "boolean" }
      },
      "allOf": [
        {
          "if": { "properties": { "is_loop": { "const": true } } },
          "then": { "required": ["start_sample", "end_sample"] }
        }
      ]
    },
    "trim": {
      "type": "object",
      "additionalProperties": false,
      "required": ["leading_silence_ms", "trailing_silence_ms",
                   "fade_in_ms", "fade_out_ms"],
      "properties": {
        "leading_silence_ms": { "type": "number", "minimum": 0 },
        "trailing_silence_ms": { "type": "number", "minimum": 0 },
        "fade_in_ms": { "type": "number", "minimum": 0 },
        "fade_out_ms": { "type": "number", "minimum": 0 }
      }
    },
    "layers": {
      "type": "array",
      "maxItems": 12,
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["id", "role", "gain_db"],
        "properties": {
          "id": { "type": "string", "pattern": "^[a-z0-9_]+$" },
          "role": {
            "enum": ["transient", "body", "tail", "mech", "debris",
                     "sub", "air", "tonal", "noise"]
          },
          "stem_file": { "type": "string" },
          "gain_db": { "type": "number", "minimum": -60, "maximum": 12 },
          "pan": { "type": "number", "minimum": -1, "maximum": 1 },
          "pitch_cents": { "type": "number", "minimum": -1200, "maximum": 1200 },
          "mute_by_default": { "type": "boolean" }
        }
      }
    },
    "variation": {
      "type": "object",
      "additionalProperties": false,
      "required": ["family_id", "index", "count"],
      "properties": {
        "family_id": { "type": "string", "pattern": "^[a-z0-9._-]+$" },
        "index": { "type": "integer", "minimum": 0 },
        "count": { "type": "integer", "minimum": 1 },
        "avoid_repeat_window": { "type": "integer", "minimum": 0 },
        "pitch_cents_range": {
          "type": "array", "minItems": 2, "maxItems": 2,
          "items": { "type": "number" }
        },
        "gain_db_range": {
          "type": "array", "minItems": 2, "maxItems": 2,
          "items": { "type": "number" }
        },
        "distance_variant": { "enum": ["none", "near", "far"] },
        "style_grade": { "enum": ["calm", "alert", "rage", "any"] }
      }
    },
    "mix": {
      "type": "object",
      "additionalProperties": false,
      "required": ["bus", "priority", "max_instances"],
      "properties": {
        "bus": { "type": "string", "pattern": "^[A-Za-z]+(/[A-Za-z]+)*$" },
        "priority": { "type": "integer", "minimum": 0, "maximum": 128 },
        "max_instances": { "type": "integer", "minimum": 1, "maximum": 64 },
        "cooldown_sec": { "type": "number", "minimum": 0 },
        "attenuation": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "min_distance": { "type": "number", "minimum": 0 },
            "max_distance": { "type": "number", "minimum": 0 },
            "curve": { "enum": ["linear", "logarithmic", "custom"] }
          }
        },
        "occlusion": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "mode": { "enum": ["none", "runtime_lpf", "baked_variants"] },
            "lpf_hz_near": { "type": "number" },
            "lpf_hz_far": { "type": "number" }
          }
        }
      }
    },
    "engine": {
      "type": "object",
      "additionalProperties": false,
      "required": ["unity"],
      "properties": {
        "unity": {
          "type": "object",
          "additionalProperties": false,
          "required": ["address", "labels", "load_type", "compression",
                       "quality", "preload", "mixer_group"],
          "properties": {
            "address": { "type": "string" },
            "labels": {
              "type": "array",
              "items": { "type": "string" }
            },
            "load_type": {
              "enum": ["DecompressOnLoad", "CompressedInMemory", "Streaming"]
            },
            "compression": {
              "enum": ["PCM", "ADPCM", "Vorbis", "MP3"]
            },
            "quality": { "type": "number", "minimum": 0, "maximum": 1 },
            "sample_rate_setting": {
              "enum": ["PreserveSampleRate", "OptimizeSampleRate", "OverrideSampleRate"]
            },
            "preload": { "type": "boolean" },
            "force_mono": { "const": false },
            "mixer_group": { "type": "string" },
            "addressables_group": { "type": "string" },
            "bundle_mode": {
              "enum": ["PackTogether", "PackSeparately", "PackTogetherByLabel"]
            }
          }
        },
        "fmod": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "event_path": { "type": "string", "pattern": "^event:/" },
            "bank": { "type": "string" },
            "bus": { "type": "string" },
            "loop_region_samples": {
              "type": "array", "minItems": 2, "maxItems": 2,
              "items": { "type": "integer" }
            }
          }
        },
        "wwise": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "event": { "type": "string", "pattern": "^Play_" },
            "bank": { "type": "string" },
            "bus": { "type": "string" },
            "switch": { "type": "string" },
            "rtpc": { "type": "array", "items": { "type": "string" } }
          }
        }
      }
    },
    "qa": {
      "type": "object",
      "additionalProperties": false,
      "required": ["passed", "checks"],
      "properties": {
        "passed": { "type": "boolean" },
        "checks": {
          "type": "object",
          "additionalProperties": {
            "type": "object",
            "additionalProperties": false,
            "required": ["status", "value"],
            "properties": {
              "status": { "enum": ["pass", "warn", "fail"] },
              "value": {},
              "threshold": {},
              "message": { "type": "string" }
            }
          }
        }
      }
    },
    "provenance": {
      "type": "object",
      "additionalProperties": false,
      "required": ["created_utc", "tool_version", "preset", "sha256"],
      "properties": {
        "created_utc": { "type": "string", "format": "date-time" },
        "tool_version": { "type": "string" },
        "preset": { "type": "string" },
        "license": { "type": "string" },
        "sha256": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
        "source_hash": { "type": "string" },
        "generator_params": { "type": "object" }
      }
    }
  }
}
```

**Örnek instance (kısaltılmış):**

```json
{
  "schema": "soundgen.sfx/1.0",
  "asset_id": "weapon.pistol.fire",
  "file": "SFX_WEAPON_PISTOL_FIRE_V01_RL.wav",
  "category": "weapon",
  "subcategory": "pistol.fire",
  "tags": ["gunshot", "9mm", "close", "dry", "layer:transient"],
  "seed": 184467,
  "style": 0.35,
  "audio": {
    "sample_rate": 48000, "bit_depth": 24, "channels": 1,
    "format": "wav_pcm", "duration_sec": 0.842, "samples": 40416
  },
  "loudness": {
    "measure": "momentary_max", "target_lufs": -14.0,
    "rel_lufs": -14.2, "momentary_max_lufs": -14.1,
    "short_term_max_lufs": -15.6, "integrated_lufs": -15.8,
    "true_peak_dbtp": -1.4, "peak_dbfs": -1.5, "plr_db": 12.7,
    "gain_applied_db": -6.8, "meter": "libebur128-compatible"
  },
  "loop": { "is_loop": false, "mode": "none", "seamless_verified": false },
  "trim": {
    "leading_silence_ms": 0.0, "trailing_silence_ms": 8.0,
    "fade_in_ms": 0.0, "fade_out_ms": 10.0
  },
  "layers": [
    { "id": "transient", "role": "transient", "gain_db": 0.0 },
    { "id": "body", "role": "body", "gain_db": -1.5 },
    { "id": "tail", "role": "tail", "gain_db": -4.0 }
  ],
  "variation": {
    "family_id": "weapon.pistol.fire", "index": 0, "count": 6,
    "avoid_repeat_window": 3,
    "pitch_cents_range": [-30, 30], "gain_db_range": [-1.5, 1.5],
    "distance_variant": "near", "style_grade": "any"
  },
  "mix": {
    "bus": "SFX/Weapon", "priority": 110, "max_instances": 6,
    "cooldown_sec": 0.05,
    "attenuation": { "min_distance": 3, "max_distance": 120, "curve": "logarithmic" },
    "occlusion": { "mode": "runtime_lpf", "lpf_hz_near": 20000, "lpf_hz_far": 1400 }
  },
  "engine": {
    "unity": {
      "address": "sfx/weapon/pistol/fire/v01",
      "labels": ["sfx", "cat:weapon", "family:weapon.pistol.fire", "bank:core"],
      "load_type": "DecompressOnLoad", "compression": "ADPCM",
      "quality": 1.0, "sample_rate_setting": "PreserveSampleRate",
      "preload": true, "force_mono": false,
      "mixer_group": "SFX/Weapon", "addressables_group": "Core_SFX",
      "bundle_mode": "PackTogether"
    },
    "fmod": {
      "event_path": "event:/SFX/Weapon/Pistol/Fire",
      "bank": "SFX_Core", "bus": "bus:/SFX/Weapon",
      "loop_region_samples": [0, 40416]
    },
    "wwise": {
      "event": "Play_SFX_Weapon_Pistol_Fire",
      "bank": "SFX_Core", "bus": "SFX/Weapon",
      "switch": "Weapon_Type/Pistol",
      "rtpc": ["Distance", "Occlusion"]
    }
  },
  "qa": {
    "passed": true,
    "checks": {
      "true_peak": { "status": "pass", "value": -1.4, "threshold": -1.0 },
      "dc_offset": { "status": "pass", "value": 0.00004, "threshold": 0.001 },
      "naming": { "status": "pass", "value": true }
    }
  },
  "provenance": {
    "created_utc": "2026-10-24T12:00:00Z",
    "tool_version": "0.1.0",
    "preset": "weapon_pistol_fire",
    "license": "proprietary",
    "sha256": "9f2c0e6a1b7d4c8e5f0a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f70"
  }
}
```

---

## 4. Unity entegrasyonu

### 4.1 Import akışı

1. Generator `Assets/Audio/...` altına WAV + `.wav.sfx.json` yazar.
2. `AudioAssetPostprocessor : AssetPostprocessor` sidecar'ı okur, `AudioImporter` ayarlarını **platform bazlı** uygular, `SaveAndReimport` çağırır.
3. `AddressablesManifestBuilder` tüm sidecar'ları tarar; `address`, `labels`, `group`, `bundle_mode` alanlarını `AddressableAssetSettings` üzerinden uygular.
4. `SfxCatalogBuilder` `SfxEntry` ScriptableObject'lerini üretir (veya runtime'da JSON'dan yükler — ikisi de desteklenir; build-time üretim ECS için daha hızlıdır).

**Neden AssetPostprocessor:** Elle Inspector ayarlamak ölçeklenmez ve "bir asset yanlış ayarla build'e girdi" hatası sessizdir. Sidecar'ı tek doğruluk kaynağı yapmak, ayarın da versiyonlanmasını sağlar.

```csharp
// Assets/Editor/AudioAssetPostprocessor.cs
using System.IO;
using UnityEditor;
using UnityEngine;

public sealed class AudioAssetPostprocessor : AssetPostprocessor
{
    void OnPreprocessAudio()
    {
        if (!assetPath.EndsWith(".wav")) return;
        var sidecar = assetPath + ".sfx.json";
        if (!File.Exists(sidecar)) return;

        var dto = JsonUtility.FromJson<SidecarDto>(File.ReadAllText(sidecar));
        var importer = (AudioImporter)assetImporter;

        importer.forceToMono = false;      // generator zaten doğru kanalda üretir
        importer.loadInBackground = dto.engine.unity.load_type == "Streaming";

        var baseline = new AudioImporterSampleSettings
        {
            loadType = ParseLoadType(dto.engine.unity.load_type),
            compressionFormat = ParseFormat(dto.engine.unity.compression),
            quality = Mathf.Clamp01(dto.engine.unity.quality),
            sampleRateSetting = AudioSampleRateSetting.PreserveSampleRate,
            preloadAudioData = dto.engine.unity.preload
        };
        importer.defaultSampleSettings = baseline;

        // Mobil: kısa SFX -> ADPCM, uzun ambience -> Vorbis
        var mobile = baseline;
        if (dto.audio.duration_sec >= 5f)
        {
            mobile.compressionFormat = AudioCompressionFormat.Vorbis;
            mobile.loadType = AudioClipLoadType.CompressedInMemory;
            mobile.quality = 0.6f;
        }
        importer.SetOverrideSampleSettings(BuildTargetGroup.Android, mobile);
        importer.SetOverrideSampleSettings(BuildTargetGroup.iOS, mobile);
        importer.SetOverrideSampleSettings(BuildTargetGroup.Standalone, baseline);
    }

    static AudioClipLoadType ParseLoadType(string s) => s switch
    {
        "Streaming" => AudioClipLoadType.Streaming,
        "CompressedInMemory" => AudioClipLoadType.CompressedInMemory,
        _ => AudioClipLoadType.DecompressOnLoad
    };

    static AudioCompressionFormat ParseFormat(string s) => s switch
    {
        "PCM" => AudioCompressionFormat.PCM,
        "ADPCM" => AudioCompressionFormat.ADPCM,
        "MP3" => AudioCompressionFormat.MP3,
        _ => AudioCompressionFormat.Vorbis
    };
}

// JsonUtility yalnızca ihtiyaç duyulan alt kümeyi okur; fazla alanlar yok sayılır.
// Tam şema doğrulaması build adımında Newtonsoft/System.Text.Json ile yapılır.
[System.Serializable] public sealed class SidecarDto { public AudioDto audio; public EngineDto engine; }
[System.Serializable] public sealed class AudioDto { public float duration_sec; }
[System.Serializable] public sealed class EngineDto { public UnityDto unity; }
[System.Serializable] public sealed class UnityDto
{
    public string load_type, compression, address, mixer_group;
    public string addressables_group, bundle_mode;
    public float quality;
    public bool preload;
}
```

### 4.2 Import ayarları (platform bazlı)

| Kategori | Android / iOS | Windows (Standalone) | Load Type | Quality |
|---|---|---|---|---|
| Weapon, gore, impact (< 2 s) | **ADPCM** | **PCM** | Decompress On Load | 1.0 |
| Foley, footstep (< 0.5 s) | **ADPCM** | **PCM** | Decompress On Load | 1.0 |
| UI (< 0.4 s) | **PCM** | **PCM** | Decompress On Load | — |
| Zombie vocal (0.5–3 s) | **ADPCM** | **ADPCM** | Decompress On Load | 1.0 |
| Ambience loop (20–60 s) | **Vorbis 0.5–0.6** | **Vorbis 0.7** | Compressed In Memory | 0.55 |
| Propeller/engine loop (2–8 s) | **ADPCM** | **ADPCM** | Decompress On Load | 1.0 |
| Music stinger (< 8 s) | Vorbis 0.6 | Vorbis 0.8 | Compressed In Memory | 0.7 |
| Music (uzun) | Vorbis 0.6 | Vorbis 0.8 | **Streaming** | 0.7 |

**Notlar:**

- `Force To Mono` **kapalı** kalır. Unity'nin Force To Mono'su downmix sonrası **peak-normalize** eder; bu, asset'in loudness hedefini bozar ve stereo genişliği geri döndürülemez şekilde yok eder. Doğru kanal sayısı generator'ın sorumluluğundadır.
- `Sample Rate Setting = PreserveSampleRate`. `OptimizeSampleRate` yalnızca ambience'ta **MAY**; weapon/UI/loop'ta asla (faz/loop süresi hassasiyeti).
- `Preload Audio Data`: kısa SFX'te **true**, ambience/music'te **false**. Preload kapalıysa ilk `Play` gecikme üretir; `AudioClip.LoadAudioData()` ile önceden yüklenmelidir.
- Unity dokümantasyonu: Decompress On Load, Vorbis'te yaklaşık **10×**, ADPCM'de yaklaşık **3.5×** bellek genişlemesi yapar. Streaming'in clip başına ~200 KB sabit overhead'i vardır — bu yüzden 100 kısa SFX asla streaming yapılmaz.
- Unity'nin desteklediği kaynak formatlar: `.wav`, `.aif`, `.mp3`, `.ogg`, `.xm`, `.mod`, `.it`, `.s3m`. Bizim sözleşmemiz yalnızca `.wav` kullanır.

### 4.3 Addressables gruplaması

| Group | İçerik | Bundle Mode | Compression | Yükleme |
|---|---|---|---|---|
| `Core_SFX` | weapon, gore, ui, foley | **Pack Together** | LZ4 | Boot'ta, kalıcı |
| `Zombie_Voices` | VOX / zombie vocal family'leri | Pack Together (family başına label) | LZ4 | Level yüklemesinde |
| `Ambience_<Level>` | ambience bed'leri | **Pack Separately** | LZ4 | Level akışı |
| `Music` | stinger + loop | Pack Separately | LZ4 | İsteğe bağlı stream |
| `Props_<Level>` | crate, confetti, whoosh | Pack Together | LZ4 | Level |

**Label'lar (sidecar'dan otomatik):**

- `sfx` — tüm SFX
- `cat:weapon`, `cat:ui`, `cat:ambience` — kategori
- `family:weapon.pistol.fire` — varyasyon ailesi (runtime seçim için)
- `bank:core`, `bank:level_swamp` — kıta/bölüm
- `loop` — loop asset'leri
- `platform:mobile` — mobilde düşürülecek asset'ler (örn. 2. near varyantı)

Çoklu label sorgusu **`MergeMode.Intersection`** ile yapılır: `Addressables.LoadAssetsAsync<AudioClip>(new[]{"family:weapon.pistol.fire","bank:core"}, null, MergeMode.Intersection)`.

**Manifest → label eşlemesi:** Build adımı her sidecar için `engine.unity.labels` dizisini olduğu gibi uygular; label'lar **türetilmez, yazılır**. Böylece "hangi asset hangi bundle'da" sorusunun cevabı JSON'da okunabilir kalır ve FMOD/Wwise build'leri aynı manifest'ten beslenebilir.

### 4.4 AudioMixer yapısı

```
Master
├── SFX
│   ├── Weapon        (duck: AMB -4 dB, 120 ms)
│   ├── Gore
│   ├── Foley
│   └── Prop
├── VOX               (zombie vocal)
├── AMB               (duck kaynağı değil, hedefi)
├── UI                (2D, spatialize kapalı)
├── MUS
└── Returns
    ├── Reverb_Small
    └── Reverb_Large
```

- Her grup için exposed parameter: `Vol_SFX`, `Vol_Weapon`, `Vol_UI`, `Vol_AMB`, `Vol_MUS`, `Vol_VOX`.
- Snapshot'lar: `Default`, `Combat`, `Menu`, `Paused`, `Deafened` (Master LPF 800 Hz, −12 dB).
- Ducking: `Weapon → AMB`, threshold tabanlı, 4 dB, attack 20 ms, release 400 ms. Zombie horde bed'i (`AMB`) silah sesiyle maskelenmemeli.
- `AudioSource.outputAudioMixerGroup` sidecar'daki `mix.bus` ile birebir eşleşir; eşleşmezse build uyarısı.

### 4.5 Unity 6 Audio Random Container (ARC)

Unity 6'da `Audio Random Container` yerleşik olarak shuffle/random, `Avoid Repeating Last`, pitch/volume randomization ve per-clip volume sunar. Bu, Wwise Random Container'ın yerel karşılığıdır.

**Ne zaman ARC:** tek bir event'in 4–12 varyasyonu varsa, asset'ler küçükse ve Addressables streaming gerekmiyorsa. ARC, kod yazmadan doğru davranışı verir.
**Ne zaman kod (aşağıdaki picker):** ECS/DOTS tarafında toplu voice yönetimi, global "no-repeat" penceresi, kategori bazlı cooldown, deterministik replay (seed) veya Addressables'tan gelen dinamik klipler gerekiyorsa.

ARC ile ilgili dikkat: `Avoid Repeating Last` değeri clip sayısından küçük olmalı; aksi hâlde tek clip'e düşer. Pitch randomization cent cinsindendir ve `AudioSource.pitch` ile toplanır — iki yerde birden randomization yapmak "çipmunk" etkisi üretir.

### 4.6 ScriptableObject katalog + ağırlıklı, tekrarsız varyasyon seçici

```csharp
using System;
using System.Collections.Generic;
using UnityEngine;

namespace Walkin.Audio
{
    [Serializable]
    public sealed class SfxVariation
    {
        public AudioClip clip;
        [Range(0f, 4f)] public float weight = 1f;
        [Range(-24f, 6f)] public float gainDb;
        [Range(-200f, 200f)] public float pitchCents;
    }

    [CreateAssetMenu(menuName = "Walkin/Audio/SFX Entry", fileName = "SFX_")]
    public sealed class SfxEntry : ScriptableObject
    {
        public string eventId = "weapon.pistol.fire";
        public string bus = "SFX/Weapon";
        [Range(0, 128)] public int priority = 100;
        [Range(1, 32)] public int maxInstances = 4;
        [Min(0f)] public float cooldownSec = 0.02f;
        [Range(0f, 200f)] public float randomPitchCents = 40f;
        [Range(0f, 6f)] public float randomGainDb = 1.5f;
        [Range(0, 16)] public int avoidRepeatWindow = 3;
        public SfxVariation[] variations = Array.Empty<SfxVariation>();
    }

    [CreateAssetMenu(menuName = "Walkin/Audio/SFX Catalog", fileName = "SfxCatalog")]
    public sealed class SfxCatalog : ScriptableObject
    {
        public SfxEntry[] entries = Array.Empty<SfxEntry>();
        Dictionary<string, SfxEntry> _map;

        public bool TryGet(string eventId, out SfxEntry entry)
        {
            if (_map == null)
            {
                _map = new Dictionary<string, SfxEntry>(entries.Length, StringComparer.Ordinal);
                foreach (var e in entries)
                    if (e != null && !string.IsNullOrEmpty(e.eventId)) _map[e.eventId] = e;
            }
            return _map.TryGetValue(eventId, out entry);
        }

#if UNITY_EDITOR
        void OnValidate()
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var e in entries)
                if (e != null && !seen.Add(e.eventId))
                    Debug.LogError($"Duplicate eventId '{e.eventId}' in {name}", this);
        }
#endif
    }

    /// Allocation-free, deterministik, "hemen tekrar yok" seçici.
    /// ECS tarafından managed system içinde çağrılabilir; MonoBehaviour gerektirmez.
    public sealed class SfxVariationPicker
    {
        readonly Dictionary<string, int[]> _history = new(StringComparer.Ordinal);
        readonly System.Random _rng;

        public SfxVariationPicker(int seed) => _rng = new System.Random(seed);

        public int Pick(SfxEntry entry, out float gainDb, out float pitchCents)
        {
            gainDb = 0f;
            pitchCents = 0f;
            var n = entry.variations.Length;
            if (n == 0) return -1;
            if (n == 1) { Apply(entry, entry.variations[0], ref gainDb, ref pitchCents); return 0; }

            var window = Mathf.Clamp(entry.avoidRepeatWindow, 0, n - 1);
            if (!_history.TryGetValue(entry.eventId, out var hist) || hist.Length != window)
            {
                hist = new int[window];
                for (var i = 0; i < window; i++) hist[i] = -1;
                _history[entry.eventId] = hist;
            }

            var idx = WeightedIndex(entry);
            for (var attempt = 0; attempt < 8 && InWindow(hist, idx); attempt++)
                idx = WeightedIndex(entry);

            if (InWindow(hist, idx))            // tüm adaylar pencerede: en eskiyi seç
            {
                idx = hist[window - 1];
                for (var i = window - 1; i > 0; i--) hist[i] = hist[i - 1];
                hist[0] = idx;
            }
            else if (window > 0)
            {
                for (var i = window - 1; i > 0; i--) hist[i] = hist[i - 1];
                hist[0] = idx;
            }

            Apply(entry, entry.variations[idx], ref gainDb, ref pitchCents);
            return idx;
        }

        int WeightedIndex(SfxEntry entry)
        {
            var total = 0f;
            foreach (var v in entry.variations) total += Mathf.Max(0f, v.weight);
            if (total <= 0f) return _rng.Next(entry.variations.Length);
            var r = (float)_rng.NextDouble() * total;
            for (var i = 0; i < entry.variations.Length; i++)
            {
                r -= Mathf.Max(0f, entry.variations[i].weight);
                if (r <= 0f) return i;
            }
            return entry.variations.Length - 1;
        }

        static bool InWindow(int[] hist, int idx)
        {
            for (var i = 0; i < hist.Length; i++) if (hist[i] == idx) return true;
            return false;
        }

        static void Apply(SfxEntry entry, SfxVariation v, ref float gainDb, ref float pitchCents)
        {
            gainDb = v.gainDb + UnityEngine.Random.Range(-entry.randomGainDb, entry.randomGainDb);
            pitchCents = v.pitchCents + UnityEngine.Random.Range(-entry.randomPitchCents, entry.randomPitchCents);
        }

        /// cents -> AudioSource.pitch (1.0 = 0 cent)
        public static float CentsToPitch(float cents) => Mathf.Pow(2f, cents / 1200f);
    }
}
```

**ECS notu:** `SfxEntry`/`SfxCatalog` saf veridir; `Pick` allocation üretmez (dictionary girişleri ısındıktan sonra). DOTS tarafında bir `AudioEventQueue : NativeQueue<AudioEventRequest>` doldurulur, managed `AudioPlaybackSystem` bu kuyruğu boşaltıp `AudioSource.PlayOneShot` çağırır. Böylece ECS job'ları managed audio API'sine dokunmaz.

---

## 5. Middleware hazırlığı (FMOD / Wwise)

Amaç: bugün Unity ile çalışan asset'lerin, yarın FMOD/Wwise'a **yeniden tasarlanmadan** taşınabilmesi. Bunun tek yolu, "oyun motorunun yapacağı işi" ses dosyasına gömmemektir.

### 5.1 Ayrım: baked vs runtime

| Katman | Nerede yapılır | Neden |
|---|---|---|
| Transient, body, tail, mech, debris | **Baked** (stem'ler) | Ses tasarımı kararı; değişmez |
| Loudness normalizasyonu | **Baked** (gain) | Teslim standardı |
| Kategori dengelemesi | Runtime (bus) | Mix değişebilir |
| Distance attenuation | Runtime (curve) | Zaten engine işi |
| Occlusion LPF / reverb send | Runtime | Geometriye bağlı |
| Pitch/volume randomization | Runtime | Anti-repetition |
| Reverb, delay, modulation | Runtime (send) | Middleware'de yeniden ayarlanabilir |

**Kural:** Reverb tail'i asset'e gömmek, o sesin farklı mekânlarda kullanılmasını imkânsız kılar. Kuru (dry) versiyon **MUST**, ıslak (wet) versiyon `_WET` flag'i ile **MAY**.

### 5.2 Stem'ler ve gain staging

- Her efekt `layers[]` dizisinde tanımlı stem'lere ayrılabilir: `transient`, `body`, `tail`, `mech`, `debris`, `sub`, `air`, `tonal`, `noise`.
- Stem'ler **aynı uzunlukta** ve **sample 0'a hizalı** olmalıdır (ör. 48 000 sample). Böylece middleware'de tek timeline'a dizilebilir.
- Stem gain'leri toplamı parent'ın loudness'ını ±0.5 LU içinde vermelidir; her stem ≤ −6 dBTP.
- Stem dosyaları `_LY_<ROLE>` flag'i ile ayrı dosyadır; parent WAV ayrıca teslim edilir (stem'ler parent'ın yerine geçmez).
- Sample rate aynı, kanal sayısı aynı, dither yok (24-bit).

### 5.3 Event isimlendirme ve bank yapısı

| Kavram | FMOD | Wwise |
|---|---|---|
| Event | `event:/SFX/Weapon/Pistol/Fire` | `Play_SFX_Weapon_Pistol_Fire` |
| Bank | `SFX_Core` | `SFX_Core` |
| Bus | `bus:/SFX/Weapon` | `SFX/Weapon` |
| Varyasyon | aynı event içinde multi-instrument / random | Random Container |
| Loop | timeline loop region (`loop_region_samples`) | Sound SFX + `smpl` |
| Parametre | `Distance`, `Occlusion` (built-in) | RTPC `Distance`, `Occlusion` |
| Switch | — | `Weapon_Type/Pistol` |

**Şimdi yazılması gereken ek metadata (sonradan eklemek pahalıdır):**

- `mix.bus` — middleware bus yolu (bugün AudioMixer grubu, yarın FMOD `bus:/`).
- `mix.attenuation` — min/max distance, curve tipi. Middleware'de aynı curve'ü kurmak için.
- `mix.occlusion` — `runtime_lpf` ise kesim frekansları; `baked_variants` ise varyant dosyaları.
- `variation.family_id` + `count` — Random Container'ı otomatik kurmak için.
- `variation.pitch_cents_range` — middleware randomization aralığı.
- `loop.start_sample/end_sample` — loop region'ı birebir taşımak için.
- `layers[]` — multi-track event'e dönüştürmek için.
- `engine.fmod` / `engine.wwise` blokları — event adı, bank, switch, RTPC isimleri.

Bu alanlar bugün Unity tarafından okunmuyorsa bile **yazılmalıdır**; maliyeti sıfırdır, geçiş maliyetini saatlerden dakikalara indirir.

---

## 6. Oyun içi varyasyon stratejisi

### 6.1 Kaç varyasyon yeter?

| Kullanım sıklığı | Önerilen varyasyon | Örnek |
|---|---|---|
| Tek seferlik / anlatı | 1 | level-up stinger, boss intro |
| Dakikada < 1 | 3 | crate open, kapı |
| Dakikada 1–10 | 5–6 | gunshot, gore impact |
| Saniyede > 1 | 8–12 | footstep, shell drop, zombie vocal |
| Horde bed | 8–12 + loop katmanı | kalabalık |

Kulak, 1→3 ve 3→5 geçişlerinde tekrar algısında büyük düşüş yaşar; 8'in üzerinde kazanç hızla azalır ve bellek maliyeti doğrusal artar. **5 varyasyon + ±60 cent pitch + ±1.5 dB gain ≈ 20+ algısal olarak farklı olay** üretir. Yani varyasyon sayısını artırmadan önce randomization'ı kullan.

### 6.2 Pitch randomization limitleri

| Kategori | Aralık (cent) | Not |
|---|---|---|
| UI | 0 (±5) | Sabit olmalı; marka sesi |
| Weapon | ±25…40 | ±50'de oyuncaklaşır |
| Gore / impact | ±50…80 | |
| Foley / footstep | ±80…120 | En yüksek tolerans |
| Zombie vocal | ±100…150 | + formant kayması idealdir |
| Ambience / loop | 0 | Loop'u bozar |
| Music | 0 | Ton değişir |

**Kural:** toplam sapma (asset içi `pitchCents` + runtime randomization) **±200 cent**'i geçmez. Pitch, playback rate'i değiştirdiği için süre ve tizlik birlikte değişir; 100 cent ≈ %6 süre farkı. Çok kısa UI seslerinde bu, tıklamayı "çift tık" gibi duyurabilir.

### 6.3 Layering'de faz sorunu

Aynı kaynağın iki kopyasını 5–20 ms kaydırarak katmanlamak **comb filtering** üretir (belirli frekanslarda 10+ dB çentik). Kurallar:

- Katmanlar **decorrelated** olmalı: farklı seed, farklı gürültü, farklı mesafe/mikrofon.
- İki katman asla aynı dosyanın kopyası olmaz (validator kontrol eder: katmanlar arası korelasyon < 0.9).
- Polarite inversiyonu yasak (mono'da iptal).
- Katman gecikmeleri 0 veya ≥ 25 ms; 1–20 ms aralığı yalnızca bilinçli "flam" efekti için.
- Her asset mono uyumluluk testinden geçer (bkz. §1.2).

### 6.4 Mesafe ve occlusion hazırlığı

**Varsayılan: runtime.** LPF + volume curve + reverb send, asset sayısını artırmaz.

**İstisna: `baked_variants`.** Yalnızca şu durumlarda near/far çifti üret:

- Silah sesi: uzak versiyonda tail/reverb oranı ve spektrum belirgin farklıysa (oyuncu silahı yakın, NPC silahı uzak).
- Zombi çığlığı: 30 m+ için "uzak" versiyon, yankı karakteri farklıysa.
- Pervane: kokpit (yakın, stereo) vs dışarıdan (uzak, mono).

Near/far çifti bellek maliyetini ~%30–40 artırır (tail daha uzun). Bu yüzden yalnızca "duyulabilir fark" varsa üretilir; `variation.distance_variant` alanı bunu açıkça işaretler.

Geçiş: 25–40 m aralığında near→far crossfade; iki asset'in fazı uyumlu olmalı (aynı seed'in farklı render'ı değil, aynı kaynak + farklı post).

### 6.5 Bellek bütçesi (mobile zombie oyunu)

**Temel aritmetik — 48 kHz, 16-bit, mono:**

- 48 000 sample/s × 2 byte = **96 000 B/s** = 93.75 KiB/s ≈ **5.49 MiB/dakika**.
- 100 clip × ortalama 1.2 s = 120 s = **11.5 MB** ham PCM (10.97 MiB).
- ADPCM (3.5× sabit sıkıştırma): 120 s → **3.3 MB**.
- Vorbis q≈0.4, mono 48 kHz ≈ 70 kbps → 120 s ≈ **1.05 MB** diskte; ancak Decompress On Load'da RAM'de ~10× açılır → **~10.5 MB RAM**.

**Örnek bütçe (12–16 MB audio RAM hedefi, düşük-orta Android):**

| Kalem | Adet | Süre | Format | RAM |
|---|---|---|---|---|
| Core SFX (weapon, gore, foley) | 100 | 120 s | ADPCM, Decompress On Load | 3.3 MB |
| UI | 20 | 3 s | PCM | 0.6 MB |
| Zombie vocal family | 9 | 22 s | ADPCM | 0.6 MB |
| Horde bed | 2 | 60 s | Vorbis, Compressed In Memory | 0.4 MB |
| Ambience | 4 | 160 s | Vorbis, Compressed In Memory | 1.2 MB |
| Propeller loop | 2 | 12 s | ADPCM | 0.35 MB |
| Music / stinger | 3 | 30 s | Vorbis, Streaming | ~0.1 MB + 0.2 MB overhead |
| **Toplam** | | | | **~6.8 MB** |

Kalan ~7 MB, OS/Unity audio buffer'ları, DSP ve gelecekteki içerik için headroom'dur. **Streaming overhead uyarısı:** Unity'de streaming clip başına ~200 KB sabit maliyet taşır; 100 kısa SFX'i streaming yapmak 20 MB ek yük demektir — felaket.

**Kural:** < 2 s her şey **Decompress On Load + ADPCM**; > 10 s her şey **Compressed In Memory / Streaming + Vorbis**; arada kalanlar süreye göre.

---

## 7. Zombi kalabalığı için anti-fatigue tasarımı

Kalabalık, ses tasarımının en hızlı yorulan alanıdır: aynı 4 hırıltı 200 zombiyle çalarsa oyuncu 30 saniyede "ses duvarı" duyar ve tehlike bilgisi kaybolur. SoundGen bu yüzden tek tek dosya değil, **family pack** üretir.

### 7.1 Family pack yapısı

Bir `family` (ör. `zombie.male.groan`) şunları içerir:

- **8–12 varyasyon** (`V01`…`V12`), her biri farklı seed.
- **3 stil derecesi:** `calm`, `alert`, `rage` (aynı sesin enerji ekseni).
- **2 gövde tipi** (hafif/ağır) — `MAY`, bellek varsa.
- **1 loop bed** (`AMB_ZOMBIE_HORDE_BED`) — tek instance, stereo, uzaktan gelen sürekli katman.
- **Meta:** `variation.avoid_repeat_window = 3`, `pitch_cents_range = [-120, 120]`, `gain_db_range = [-2, 2]`.

Oyun tarafı bu paketi tek bir `family_id` ile yükler; tek tek dosya adı bilmez.

### 7.2 Voice pooling

- **Önceden ayrılmış** `AudioSource` havuzu: 24 adet (horde için 8, weapon 4, gore 4, foley 4, UI 4).
- Her istek: `priority` (sidecar'dan), `max_instances` (family başına), `cooldown_sec` (emitter başına).
- Voice stealing: en düşük öncelikli + en eski + en sessiz voice atılır. UI ve oyuncu silahı asla atılmaz (`priority ≥ 110`).
- **Global rate limit:** family başına saniyede 3 yeni voice (token bucket). 200 zombi aynı anda hırlarsa sadece 3'ü gerçek voice alır; kalanı sanal (virtual) kalır.
- Unity varsayılanı `Max Real Voices = 32`, `Max Virtual Voices` daha yüksek olmalı. PC'de 48–64 gerçek voice makul; mobilde 32'de kalınmalı.

### 7.3 Instance başına jitter

| Parametre | Aralık | Uygulama |
|---|---|---|
| Pitch | ±120 cent | `AudioSource.pitch = 2^(cents/1200)` |
| Volume | ±2 dB | `AudioSource.volume` |
| Start offset | 0–80 ms | Klişe "senkron" hissini kırar (loop bed'de) |
| Stereo pan (2D bed) | ±0.3 | Sadece bed'de |
| LPF | mesafeye göre 20 kHz → 1.5 kHz | `AudioLowPassFilter.cutoffFrequency` |

### 7.4 Mesafe katmanları

| Mesafe | Volume | LPF | Reverb send | Not |
|---|---|---|---|---|
| 0–8 m | 0 dB | 20 kHz | −12 dB | Tam bant |
| 8–25 m | −6 dB | 6 kHz | −6 dB | Orta |
| 25–60 m | −14 dB | 1.8 kHz | 0 dB | Uzak, tail baskın |
| > 60 m | cull | — | — | Sadece bed loop duyulur |

60 m ötesinde tek tek voice çalmak yerine **horde bed** devreye girer: bu, hem CPU hem de algısal olarak doğru çözümdür (insan kulağı 60 m'de bireysel sesleri ayırt edemez).

### 7.5 "İki kez aynı ses" kuralı

Validator ve runtime birlikte uygular:

- Aynı `family_id` içinde son **3** çalan varyasyon tekrar seçilemez (`avoid_repeat_window = 3`).
- Aynı emitter'da son **2.5 s** içinde aynı varyasyon çalamaz (emitter-local cooldown).
- Global olarak aynı clip 200 ms içinde iki kez başlatılamaz (double-trigger koruması).

---

## 8. QA / validasyon checklist

Her asset, `qa.checks` altında bu kontrollerin sonucunu taşır. `fail` → asset karantina; `warn` → build uyarısı ama geçer.

| # | Kontrol | Yöntem | Eşik | Seviye |
|---|---|---|---|---|
| 1 | Dosya formatı | RIFF header, codec = PCM | 48 kHz, 16/24/32-bit, 1–2 kanal | FAIL |
| 2 | Kanal politikası | `category` → beklenen kanal | Tablo §1.2 | FAIL |
| 3 | Süre | `samples / 48000` | Kategori aralığı (§2.3) | FAIL |
| 4 | NaN / Inf | Örnek taraması | 0 adet | FAIL |
| 5 | Clipping | \|x\| ≥ 0.9999 sayısı | 0 adet | FAIL |
| 6 | Sample peak | max\|x\| | ≤ −1.0 dBFS (−3 loop) | FAIL |
| 7 | True peak | 4× oversample (BS.1770) | ≤ −1.0 dBTP (−3 loop) | FAIL |
| 8 | DC offset | Ortalama | \|mean\| ≤ 0.001 | FAIL |
| 9 | DC drift | 100 ms blok ortalamaları | max − min ≤ 0.002 | WARN |
| 10 | Başlangıç sessizliği | İlk −60 dBFS üstü örnek | ≤ 2 ms | FAIL |
| 11 | Bitiş sessizliği | Son −60 dBFS üstü örnek | ≤ 250 ms (loop: 0) | WARN |
| 12 | Fade-out varlığı | Son 5 ms enerji eğimi | −∞'a yakınsama | WARN |
| 13 | Loudness (REL) | BS.1770, gatesiz | Hedef ± 1.5 LU | FAIL |
| 14 | PLR | TP − REL | Kategori aralığı | WARN |
| 15 | Loop sürekliliği | \|x[N−1] − x[0]\| | ≤ 0.01 FS | FAIL |
| 16 | Loop RMS sürekliliği | Son/ilk 10 ms RMS farkı | ≤ 1.5 dB | FAIL |
| 17 | Loop spektral akış | 1/3-oktav, son vs ilk 20 ms | ≤ 6 dB/band | WARN |
| 18 | Loop dropout | 200 ms pencere taraması | hiçbiri < −60 dBFS | FAIL |
| 19 | Loop periyot | `blade_pass_hz` × süre | tam sayı (±0.01) | WARN |
| 20 | Mono uyumluluk | L/R korelasyonu, sum | corr > 0.2, sum ≥ −3 dB | FAIL (stereo) |
| 21 | Katman korelasyonu | Stem çiftleri | < 0.9 | WARN |
| 22 | Katman toplamı | Stem sum vs parent | ±0.5 LU | WARN |
| 23 | İsimlendirme | Regex §3.1 | eşleşme | FAIL |
| 24 | Sidecar şeması | JSON Schema doğrulama | geçerli | FAIL |
| 25 | Metadata tutarlılığı | Ölçülen vs JSON | süre ±1 ms, loop ±0 sample, LUFS ±0.1 | FAIL |
| 26 | Determinizm | Aynı seed → sha256 | 24-bit'te birebir | WARN |
| 27 | Sessiz dosya | Tüm dosya max | > −40 dBFS | FAIL |
| 28 | Süreklilik (one-shot) | Son 10 ms'te sıfıra fade | ≥ −80 dBFS'e iniş | WARN |
| 29 | Bus eşleşmesi | `mix.bus` ∈ AudioMixer | mevcut | FAIL |
| 30 | Addressables | `address` tekil, label'lar mevcut | çakışma yok | FAIL |

**Loop süreklilik testi (pseudo-code):**

```python
def loop_seam_ok(x, sr=48000):
    d0 = abs(x[-1] - x[0])
    rms_a = rms(x[: int(0.010 * sr)])
    rms_b = rms(x[-int(0.010 * sr):])
    rms_delta = abs(20 * log10((rms_b + 1e-9) / (rms_a + 1e-9)))
    flux = max_spectral_flux_delta(x[: int(0.020 * sr)], x[-int(0.020 * sr):])
    return d0 <= 0.01 and rms_delta <= 1.5 and flux <= 6.0
```

---

## 9. Export preset listesi

| Ayar | Unity Mobile | Unity PC | FMOD | Wwise | Raw |
|---|---|---|---|---|---|
| Container | WAV PCM | WAV PCM | WAV PCM | WAV PCM | WAV (RF64) |
| Sample rate | 48 000 | 48 000 | 48 000 | 48 000 | 48 000 |
| Bit depth | 16-bit | 24-bit | 24-bit | 24-bit | 32-bit float |
| Kanal | politikaya göre | politikaya göre | politikaya göre | politikaya göre | ham |
| Dither | TPDF | yok | yok | yok | yok |
| Loudness | Kategori hedefi (gain-only) | Kategori hedefi | Kategori hedefi | Kategori hedefi | **uygulanmaz** |
| True peak | ≤ −1 dBTP (loop −3) | ≤ −1 dBTP | ≤ −1 dBTP | ≤ −1 dBTP | sınırsız |
| Loop | whole_file + `smpl` | whole_file + `smpl` | timeline region + `smpl` | `smpl` + loop flag | `smpl` |
| Naming | §3.1 | §3.1 | §3.1 | §3.1 | §3.1 + `_RAW` |
| Sidecar | `.wav.sfx.json` | `.wav.sfx.json` | + `fmod` bloğu | + `wwise` bloğu | tam JSON |
| Stem'ler | sadece gerekiyorsa | evet (arşiv) | evet (`_LY_`) | evet (`_LY_`) | evet |
| Unity import | ADPCM / Vorbis | PCM / Vorbis | — | — | — |
| Ek | — | — | `event:/` yolu zorunlu | `Play_` prefix zorunlu | `provenance.sha256` |

**Export sırası (deterministik):** render → trim → loop wrap → loudness normalize (gain) → true-peak doğrula → fade → dither → yaz → ölç → sidecar → QA. Dither en sonda; sonrasında hiçbir işlem yapılmaz.

---

## 10. Kaynaklar

1. Unity — Audio Clip Import Settings reference: https://docs.unity3d.com/6000.0/Documentation/Manual/class-AudioClip.html
2. Unity — Audio file compression in Unity: https://docs.unity3d.com/6000.0/Documentation/Manual/AudioFiles-compression.html
3. Unity — `AudioImporter` (Scripting API): https://docs.unity3d.com/6000.0/Documentation/ScriptReference/AudioImporter.html
4. Unity — `AudioImporterSampleSettings` (Scripting API): https://docs.unity3d.com/6000.0/Documentation/ScriptReference/AudioImporterSampleSettings.html
5. Unity — Audio project settings (`Max Real/Virtual Voices`, DSP buffer): https://docs.unity3d.com/6000.0/Documentation/Manual/class-AudioManager.html
6. Unity — Audio Mixer window: https://docs.unity3d.com/6000.0/Documentation/Manual/AudioMixer.html
7. Unity — Audio Random Container reference: https://docs.unity3d.com/6000.0/Documentation/Manual/AudioRandomContainer-UI.html
8. Unity — Addressables: Label assets: https://docs.unity3d.com/Packages/com.unity.addressables@2.7/manual/Labels.html
9. Unity — Addressables API (`Addressables` class): https://docs.unity3d.com/Packages/com.unity.addressables@2.7/api/UnityEngine.AddressableAssets.Addressables.html
10. Unity Discussions — Max Real Voices 32 sınırı: https://discussions.unity.com/t/max-real-voices-always-limited-to-32/910855
11. ITU-R — BS.1770-5, Algorithms to measure audio programme loudness and true-peak audio level: https://www.itu.int/rec/R-REC-BS.1770-5-202311-I/en
12. EBU — R128 Loudness Recommendation: https://tech.ebu.ch/publications/r128
13. AES — TD1004.1.15-10, Recommendation for Loudness of Audio Streaming and Network File Playback: https://aes.org/wp-content/uploads/2024/01/AESTD1004_1_15_10.pdf
14. AES — TD1008 (2021), Recommendations for Loudness of Internet Audio Streaming and On-Demand Distribution: https://www.aes.org/technical/documentDownloads.cfm?docID=731
15. Stephen Schappler — "Listening for Loudness in Video Games" (Sony ASWG-R001 özeti: konsol −23 LUFS, taşınabilir −18 LUFS, −1 dBTP): https://www.stephenschappler.com/2013/07/26/listening-for-loudness-in-video-games/
16. Flux — Loudness metering, gating (absolute −70 LUFS, relative −10 LU): https://doc.flux.audio/mira/Metering_Loudness.html
17. Critical Listening Lab — Game Audio & Interactive Media Specs: https://www.criticallisteninglab.com/en/learn/streaming/video-games
18. Austin Haynes — How Game Engines Handle Loop Points (Unity `smpl` davranışı): https://www.austinhaynesmusic.com/game-engine-loop-points
19. Audiokinetic — Wwise Creating Random Containers (shuffle / Limit Repetition): https://www.audiokinetic.com/en/public-library/2025.1.10_9233/?source=Help&id=creating_random_container
20. Audiokinetic — Random Container Property Editor: https://www.audiokinetic.com/fr/public-library/2025.1.7_9143/?source=Help&id=random_sequence_container_property_editor
21. FMOD — Studio Advanced Topics (event/bank yapısı): https://www.fmod.com/docs/2.04/studio/advanced-topics.html
22. Airwiggles — Naming Conventions for Audio in Large Projects: https://www.airwiggles.com/c/general/naming-conventions-for-audio-in-large-projects
23. Production Advice — AES TD1008 summary (−16 LUFS müzik dağıtım hedefi): https://productionadvice.co.uk/td1008/

---

## Özet (15 satır)

1. Master her zaman **48 kHz / 24-bit WAV PCM**; teslim 16-bit (Unity) veya 24-bit (middleware), dither yalnızca 16-bit'e inerken.
2. 3D her ses **mono**; stereo yalnızca ambience, confetti, müzik ve 2D pervane için.
3. Peak tavanı one-shot'ta **−1 dBTP**, loop/bed'de **−3 dBTP**, stem'de **−6 dBTP**; mix bus'ta limiter yok.
4. Başlangıç sessizliği ≤ **2 ms**, fade-in yok; bitiş fade-out 5–15 ms, tail −60 dBFS'e kadar korunur.
5. Loop sözleşmesi **whole_file**: intro yok, tam periyot, tonal loop'ta blade-pass tam katı, broadband'da 20–50 ms equal-power crossfade.
6. Loudness **REL** (BS.1770 K-weighted, gatesiz, dosya boyu) + **M-max** + **true peak** üçlüsüyle ölçülür; integrated tek başına kısa SFX için geçersizdir.
7. Hedefler: gunshot −14, gore −16, zombie vocal −18, horde bed −24, foley −26, ambience −28, UI −20, stinger −16 LUFS.
8. Isimlendirme `PREFIX_DOMAIN_SUBJECT_ACTION_Vnn_FLAGS.wav`; mantıksal anahtar `weapon.pistol.fire`; sidecar dosyası **`.wav.sfx.json`**.
9. Sidecar JSON şeması kategori, seed, style, loudness, loop, katmanlar, varyasyon, mix, engine ve QA bloklarını taşır; build'in tek doğruluk kaynağıdır.
10. Unity: `AudioImporter` ayarları sidecar'dan `AssetPostprocessor` ile uygulanır; kısa SFX ADPCM, uzun ambience Vorbis; `Force To Mono` asla açılmaz.
11. Addressables: `Core_SFX` pack-together, ambience pack-separately; label'lar `cat:`, `family:`, `bank:` önekleriyle yazılır, türetilmez.
12. Varyasyon: sık çalan olaylarda 5–6, çok sık olanlarda 8–12; pitch ±25–150 cent (UI ve müzikte 0), toplam sapma ±200 cent'i geçmez.
13. Zombi kalabalığı: 24 voice havuzu, family başına 3 voice/sn token bucket, 60 m ötesinde bireysel voice yerine horde bed.
14. Bellek: 48k/16-bit mono 5.49 MiB/dk; 100 kısa SFX PCM 11.5 MB, ADPCM 3.3 MB; streaming'in clip başına ~200 KB overhead'i vardır.
15. QA: 30 otomatik kontrol (clipping, DC, true peak, loop seam, loudness, naming, schema, determinizm) — `fail` olan asset karantinaya gider.

**Dosya yolu doğrulandı:** `C:\Users\kyusu\dev\soundGen\docs\research\04-entegrasyon.md`
