# SoundGen — Proje Brifingi

## Amaç
Bir **zombi oyunu** için ihtiyaç duyulan *tüm* ses efektlerini üreten, dinleyip anında düzenleyebilen,
profesyonel kalitede bir **prosedürel ses efekt üretim aracı**.

## Kapsam (ses kategorileri)
- Silah sesleri (ateş, mekanizma, şarjör, kovan, susturucu)
- Zombi sesleri (hırıltı, çığlık, ısırma, ölüm, kalabalık)
- Vurma / isabet / gore (et, kemik, kafa patlaması)
- Folyo / adım / çevre (rüzgâr, enkaz, kuş, uzak savaş)
- Pervane / drone / motor döngüleri
- Kasa açma (tahta gıcırtı, kilit, patlama, ödül)
- Konfeti / parti bombası / kutlama katmanları
- UI (buton, ödül, coin, level-up), whoosh ve geçiş efektleri

## Temel Ürün Gereksinimleri
1. **Anlık ses** (düşük gecikmeli, tuşa basınca çalan) — "audition" akışı
2. **Parametrik düzenleme** — dalga formu/spectrogram üzerinde görsel geri bildirim
3. **Gerçekçilik ↔ Animasyon tarzı** arasında sürekli geçiş (morph / stil makası)
4. **Varyasyon üretimi** — seed tabanlı, oyunda tekrar hissi vermeyen kümeler
5. **Oyun içine hazır export** — 48 kHz, mono/stereo, LUFS normalizasyonu, isimlendirme + metadata
6. **Katmanlı mimari** — her efekt, yeniden kullanılabilir "katman" (layer) grafiklerinden oluşur

## Kısıtlar / Varsayımlar
- Hedef platform: Windows masaüstü (geliştirme makinesi: i5-11400H, RTX 3050Ti 4GB, 16GB RAM)
- Entegrasyon hedefi: Unity (Walkin projesi) — ileride FMOD/Wwise
- Üretim tamamen **yerel** olmalı (bulut maliyeti yok); nöral modeller opsiyonel
- Arayüz Türkçe/İngilizce olabilir; kod ve dosya adları İngilizce

## Çalışma Biçimi
- Araştırma: `docs/research/*.md` (paralel subagent'lar)
- Mimari kararlar: `docs/adr/`
- Kod: `apps/` (uygulama), `packages/` (DSP çekirdeği), `docs/`
