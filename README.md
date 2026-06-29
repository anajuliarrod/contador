# Contador - Programa Anchor (Solana)

Programa **Anchor** simples que implementa um _contador_ on-chain com duas instruções:
`initialize` e `increment`. Testado com LiteSVM e deployado na **devnet**.

## Desafio

Criar um programa Anchor que implemente um _counter_ (contador) simples com duas instruções:
`initialize` e `increment`.

- [x] Escrever pelo menos um teste que valide o fluxo `initialize` → `increment`
- [x] Fazer deploy do programa na devnet
- [x] Interagir com o programa deployado (chamar `initialize` e depois `increment` pelo menos uma vez)

##  Submission

| Requisito                                       | Valor                                                                                                                                                          |
| ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Program ID (devnet)**                         | `J6VdM3QeQFL3m2Shh3PXA6DcmELLhQfUwCqbhykzAhSa`                                                                                                                  |
| **Deploy do programa (Solana Explorer)**        | https://explorer.solana.com/address/J6VdM3QeQFL3m2Shh3PXA6DcmELLhQfUwCqbhykzAhSa?cluster=devnet                                                                 |
| **Transação de `increment` (Solana Explorer)**  | https://explorer.solana.com/tx/2mQE3LP8u4qK4xpdoSefBgNGmn8UaiAfCEB17xnBPzKUNrpbTe7sNaxFbg92qrduW15Q9SGufboiPydhxLBjMh69?cluster=devnet                          |
| **Repositório público (GitHub)**                | https://github.com/anajuliarrod/contador                                                                                                                            |

### Detalhes adicionais da interação on-chain

- Conta do contador criada: [`F4v651qZWvopzoBuLBMppzRYBqRFwMKx1Z8iB3WR6ozu`](https://explorer.solana.com/address/F4v651qZWvopzoBuLBMppzRYBqRFwMKx1Z8iB3WR6ozu?cluster=devnet)
- Transação de `initialize`: [`FRznL2DAN5FxU2Kqo529jGtXt51gSk6jNm1yCqMSAoStdNNmdQAzWCwMYoTat3utXDLKgntuftB3kd4wepFgFR6`](https://explorer.solana.com/tx/FRznL2DAN5FxU2Kqo529jGtXt51gSk6jNm1yCqMSAoStdNNmdQAzWCwMYoTat3utXDLKgntuftB3kd4wepFgFR6?cluster=devnet)
- Transação do deploy: [`vFkcgx6jnoAh8Dw7Nkh7yNsSyNe2fSyvjGpgB4LWApC9QGg1HZZbQHDWPbWec8GB6FZynPpg6WEDcJNVMYQQaTN`](https://explorer.solana.com/tx/vFkcgx6jnoAh8Dw7Nkh7yNsSyNe2fSyvjGpgB4LWApC9QGg1HZZbQHDWPbWec8GB6FZynPpg6WEDcJNVMYQQaTN?cluster=devnet)
- Autoridade de upgrade / carteira: `AC1ZexQSEscByXsDA9RV74qxCJVgwF1866mTTeiroWv4`

## Estrutura

```
programs/contador/src/lib.rs                 # o programa: initialize + increment
programs/contador/tests/test_initialize.rs   # teste LiteSVM (initialize → increment)
app/interact.ts                              # script de interação na devnet
Anchor.toml                                  # configuração (devnet)
```

## Como o programa funciona

O estado vive em uma **conta**, não no programa. Cada contador é uma conta
`Contador { valor: u64, dono: Pubkey }`:

- **`initialize(valor)`** - cria a conta, grava `valor` e define `dono = signer`.
- **`increment()`** - soma 1 ao `valor` usando `checked_add` (evita overflow silencioso).
  A constraint `has_one = dono` garante que **apenas o dono** pode incrementar.

## Teste

`programs/contador/tests/test_initialize.rs` carrega o `.so` compilado em uma VM
Solana in-process (**LiteSVM**) e:

1. envia `initialize(valor = 0)` → confere `valor == 0` e `dono == usuário`;
2. envia `increment()` → confere `valor == 1`.
