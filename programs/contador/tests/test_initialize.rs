use {
    anchor_lang::{
        solana_program::{instruction::Instruction, pubkey::Pubkey, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    contador::Contador,
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

/// Carrega o `.so` compilado pelo `anchor build` numa VM Solana in-process.
fn setup() -> (LiteSVM, Pubkey) {
    let program_id = contador::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/contador.so");
    svm.add_program(program_id, bytes).unwrap();
    (svm, program_id)
}

/// Cria uma carteira nova já com saldo pra pagar rent/fees.
fn carteira_com_saldo(svm: &mut LiteSVM) -> Keypair {
    let kp = Keypair::new();
    svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    kp
}

/// Monta a tx de `initialize(valor)` — assinada pelo usuário e pela conta nova.
fn initialize_tx(
    svm: &LiteSVM,
    program_id: Pubkey,
    usuario: &Keypair,
    contador_kp: &Keypair,
    valor: u64,
) -> VersionedTransaction {
    let ix = Instruction::new_with_bytes(
        program_id,
        &contador::instruction::Initialize { valor }.data(),
        contador::accounts::Initialize {
            contador: contador_kp.pubkey(),
            usuario: usuario.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&usuario.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[usuario, contador_kp]).unwrap()
}

/// Monta a tx de `increment()` — `dono` é tanto a conta passada quanto o signatário.
fn increment_tx(
    svm: &LiteSVM,
    program_id: Pubkey,
    conta: Pubkey,
    dono: &Keypair,
) -> VersionedTransaction {
    let ix = Instruction::new_with_bytes(
        program_id,
        &contador::instruction::Increment {}.data(),
        contador::accounts::Increment {
            contador: conta,
            dono: dono.pubkey(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&dono.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[dono]).unwrap()
}

/// Lê e desserializa a conta do contador.
fn ler_contador(svm: &LiteSVM, conta: Pubkey) -> Contador {
    let raw = svm.get_account(&conta).unwrap();
    Contador::try_deserialize(&mut &raw.data[..]).unwrap()
}

#[test]
fn test_initialize_e_increment() {
    let (mut svm, program_id) = setup();
    let usuario = carteira_com_saldo(&mut svm);
    let contador_kp = Keypair::new();

    // initialize(valor = 0)
    let tx = initialize_tx(&svm, program_id, &usuario, &contador_kp, 0);
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "initialize falhou: {:?}", res.err());

    let estado = ler_contador(&svm, contador_kp.pubkey());
    assert_eq!(estado.valor, 0);
    assert_eq!(estado.dono, usuario.pubkey());

    // increment()
    let tx = increment_tx(&svm, program_id, contador_kp.pubkey(), &usuario);
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "increment falhou: {:?}", res.err());

    assert_eq!(ler_contador(&svm, contador_kp.pubkey()).valor, 1);
}

#[test]
fn test_increment_por_nao_dono_falha() {
    let (mut svm, program_id) = setup();
    let usuario = carteira_com_saldo(&mut svm);
    let atacante = carteira_com_saldo(&mut svm);
    let contador_kp = Keypair::new();

    // dono = usuario
    let tx = initialize_tx(&svm, program_id, &usuario, &contador_kp, 0);
    assert!(svm.send_transaction(tx).is_ok());

    // O atacante tenta incrementar passando a si mesmo como `dono`. A constraint
    // `has_one = dono` rejeita: contador.dono == usuario, não o atacante.
    let tx = increment_tx(&svm, program_id, contador_kp.pubkey(), &atacante);
    assert!(
        svm.send_transaction(tx).is_err(),
        "increment de não-dono deveria falhar (has_one = dono)"
    );

    // Estado intacto após a tx rejeitada.
    assert_eq!(ler_contador(&svm, contador_kp.pubkey()).valor, 0);
}

#[test]
fn test_increment_no_limite_estoura() {
    let (mut svm, program_id) = setup();
    let usuario = carteira_com_saldo(&mut svm);
    let contador_kp = Keypair::new();

    // Começa no teto do u64.
    let tx = initialize_tx(&svm, program_id, &usuario, &contador_kp, u64::MAX);
    assert!(svm.send_transaction(tx).is_ok());

    // checked_add(1) estoura -> ContadorError::Overflow, a tx falha.
    let tx = increment_tx(&svm, program_id, contador_kp.pubkey(), &usuario);
    assert!(
        svm.send_transaction(tx).is_err(),
        "increment em u64::MAX deveria falhar (Overflow)"
    );

    // Valor não mudou.
    assert_eq!(ler_contador(&svm, contador_kp.pubkey()).valor, u64::MAX);
}
