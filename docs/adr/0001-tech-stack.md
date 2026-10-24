# ADR 0001 — Teknoloji yığını: Rust + egui + cpal

**Durum:** Kabul edildi (24.10.2026)
**Karar veren:** Kullanıcı (araştırma raporu 03'ün önerisi C#/.NET idi)

## Bağlam

Araştırma raporu (`docs/research/03-stack.md`) 9 kriterli ağırlıklı matriste
C#/.NET 10 + Avalonia + NAudio'yu 4.41, C++/JUCE'yi 4.11, Rust (cpal+egui) 3.92 ile puanladı.
Farkın ana nedeni: Windows'ta ≤10 ms için WASAPI exclusive/ASIO ve NAudio 3.3'ün bunu hazır vermesi.

## Karar

**Rust + egui 0.37 + cpal 0.18** kullanılacak.

## Gerekçe

1. **Gecikme hedefi bizim mimarimizde farklı.** SoundGen gerçek zamanlı bir synth değil:
   efekti offline render edip (ölçülen 8-10 ms) tamamlanmış buffer'ı çalıyor.
   Paylaşımlı WASAPI'nin ~20-30 ms tamponu bu modelde algılanan gecikmeyi bozmuyor.
2. **DSP hot path'i Rust'ta SIMD'e açık**; ileride gerçek zamanlı moda geçilirse
   (VST3 / nih-plug) aynı çekirdek yeniden yazılmadan kullanılabilir.
3. **Tek exe, bağımlılıksız dağıtım** ve AI agent'larla hızlı iterasyon.
4. C# tarafındaki kazanç (NAudio'nun hazır WASAPI sarmalayıcısı) gerekirse
   `sg-io` içine `wasapi` crate'i eklenerek 1-2 günde telafi edilir; mimari değişmez.

## Sonuçlar

- `sg-io::Player` cpal kullanır; WASAPI exclusive gerekirse arkasına yeni bir sürücü eklenir.
- C++/JUCE'ye geçiş ancak VST3 zorunlu olursa (nih-plug alternatifi) değerlendirilir.
- Risk: egui'nin 0.37 Ui-first API'si yeni; sürüm yükseltmelerinde kırılma olabilir.
  `Cargo.lock` commit edilir, yükseltmeler ayrı PR'da yapılır.
