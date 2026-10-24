# ADR 0002 — Offline render + buffer oynatma (gerçek zamanlı grafik yok)

**Durum:** Kabul edildi (24.10.2026)

## Bağlam

Ses efekti üreten bir araçta "anında duyma" iki şekilde yapılabilir:

1. Gerçek zamanlı DSP grafiği (node'lar audio callback içinde çalışır).
2. Efekti offline render edip bitmiş buffer'ı çalmak.

## Karar

**v1'de (2).** Parametre değişince efekt yeniden render edilir (40 ms debounce) ve
cpal çıkışına verilir.

## Gerekçe

- Ölçüm: 1-2 saniyelik efekt 8-10 ms'de render ediliyor; 60 fps bir arayüzde
  kullanıcı değişimi anında duyuyor.
- Audio thread'de kilit/tahsis/voice-stealing yok → tık sesi ve çökme riski yok.
- Aynı kod yolu toplu export ve QA için de kullanılıyor → ses ile dosya birebir aynı.
- Deterministik: aynı tohum aynı WAV'ı verir.

## Sonuçlar

- Gerçek zamanlı otomasyon (LFO ile canlı modülasyon) yok; gerekirse v0.4'te
  `sg-dsp` primitifleri yeniden kullanılarak gerçek zamanlı bir motor eklenir.
- Çok uzun ambiyans bedleri (30 s+) render süresini artırır; `render_length()` 30 s ile sınırlı.
- Loop'lar süre matematiğiyle hizalanır; seam doğrulaması testlerle yapılır.
