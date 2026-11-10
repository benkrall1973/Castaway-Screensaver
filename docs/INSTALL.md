# Instalação de Castaway_Screensaver

## Windows

1. Baixe `Castaway_Screensaver-Windows-x86_64.zip` e `SHA256SUMS.txt` da
   [release atual](https://github.com/benkrall1973/Castaway-Screensaver/releases/latest).
2. Confira o SHA-256 do ZIP antes de extrair: `Get-FileHash .\Castaway_Screensaver-Windows-x86_64.zip -Algorithm SHA256`.
3. Extraia para uma pasta permanente e mantenha EXE, SCR e INI juntos.
4. Coloque os dados originais em `data/` ao lado dos programas. Use uma instalação
   ou arquivo original compatível; o instalador antigo não precisa ser executado.
5. Abra `Castaway_Screensaver.exe` para configurar e testar. Clique com o botão
   direito em `Castaway_Screensaver.scr` e escolha **Instalar**.

Não mova apenas o SCR para System32. Se mover a pasta, instale o SCR novamente.
O botão de configurações abre o EXE; as preferências ficam em `Castaway_Screensaver.ini`.
Os binários ainda não têm assinatura Authenticode. Verifique origem e checksum.

## Linux e macOS

Extraia o arquivo do seu sistema e execute `./castaway_screensaver --data <pasta>`.
O pacote `.saver` para macOS usa `Castaway_Screensaver.saver`; coloque os dados em
`~/Library/Application Support/Castaway_Screensaver/`. A interface nativa de
configuração Windows não está disponível no macOS.

O pacote web é `Castaway_Screensaver-web.zip`. Sirva a pasta por HTTP e abra
`index.html`; escolha seus arquivos originais no navegador. Nenhum dado é enviado.

## Compilar e verificar

```text
cargo build --locked --release -p castaway_screensaver
cargo test --locked --workspace --all-features
```

O Rust 1.99.1 está fixado no repositório. Veja o [guia Windows](BUILD-WINDOWS.md)
e as [instruções principais](../INSTALL.md). Para builds desktop com dados
embutidos, use `CASTAWAY_SCREENSAVER_EMBED_DATA` e `--features embed-data`.
O motor web mantém a variável interna `WILSON_EMBED_DATA`.

O código é GPL-3.0-or-later e mantém os créditos do Wilson Reborn. Os dados
originais e materiais de referência têm direitos separados dessa licença.
