# Castaway_Screensaver

Reprodução moderna de Johnny Castaway, baseada no [Wilson Reborn de Antigerme](https://github.com/antigerme/wilson-reborn).

Baixe o pacote atual em [Releases](https://github.com/benkrall1973/Castaway_Screensaver/releases/latest).
No Windows, extraia a pasta inteira, mantenha EXE, SCR e INI juntos e coloque seus
dados originais em `data/`. Abra `Castaway_Screensaver.exe` para configurar;
clique com o botão direito em `Castaway_Screensaver.scr` e escolha **Instalar**.
Não copie apenas o SCR para System32. Veja [instalação](docs/INSTALL.md).

O programa procura dados em `--data <pasta-ou-zip>`, na variável
`CASTAWAY_SCREENSAVER_DATA_DIR`, no diretório atual e ao lado do executável.
O arquivo de preferências é `Castaway_Screensaver.ini`. Para atualizar uma
instalação anterior, faça backup e copie o INI anterior com esse nome antes de abrir.

Este repositório contém dados originais de referência e material de análise.
Esses arquivos têm direitos separados do código GPL-3.0-or-later. Os pacotes
binários publicados não incluem os dados originais.

## Compilar

Use o Rust 1.99.1 fixado em `rust-toolchain.toml`. No Windows, instale as
ferramentas C++ do Visual Studio e use Developer PowerShell com o toolchain MSVC.

```text
cargo build --locked --release -p castaway_screensaver
cargo run --locked -p castaway_screensaver -- --windowed --data <pasta>
```

Veja o [guia Windows](docs/BUILD-WINDOWS.md), o [README em inglês](README.md)
e a [base de engenharia herdada](docs/knowledge-base/README.md).
