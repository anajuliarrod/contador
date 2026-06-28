use {
    anchor_lang::system_program::ID::program as system_program,
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_initialize_e_increment() {
    let program_id = contador::id();
    let mut svm = LiteSVM::new();

    // carrega o .so compilado pelo `anchor build`
    let bytes = include_bytes!("../../../target/deploy/contador.so");
    svm.add_program(program_id, bytes).unwrap();

    // o usuário (paga o rent e assina)
    let usuario = Keypair::new();
    svm.airdrop(&usuario.pubkey(), 1_000_000_000).unwrap();

    // a conta do contador é um Keypair novo — ela assina a própria criação
    let contador_kp = Keypair::new();

    // ---------- initialize(valor = 0) ----------
    let init_ix = Instruction::new_with_bytes(
        program_id,
        &contador::instruction::Initialize { valor: 0 }.data(),
        contador::accounts::Initialize {
            contador: contador_kp.pubkey(),
            usuario: usuario.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[init_ix], Some(&usuario.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[&usuario, &contador_kp], // ambos assinam
    )
    .unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "initialize falhou: {:?}", res.err());

    // lê a conta e confere valor == 0
    let conta = svm.get_account(&contador_kp.pubkey()).unwrap();
    let estado = Contador::try_deserialize(&mut &conta.data[..]).unwrap();
    assert_eq!(estado.valor, 0);
    assert_eq!(estado.dono, usuario.pubkey());

    // ---------- increment() ----------
    let inc_ix = Instruction::new_with_bytes(
        program_id,
        &contador::instruction::Increment {}.data(),
        contador::accounts::Increment {
            contador: contador_kp.pubkey(),
            dono: usuario.pubkey(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[inc_ix], Some(&usuario.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[&usuario], // só o dono assina o increment
    )
    .unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "increment falhou: {:?}", res.err());

    // confere valor == 1
    let conta = svm.get_account(&contador_kp.pubkey()).unwrap();
    let estado = Contador::try_deserialize(&mut &conta.data[..]).unwrap();
    assert_eq!(estado.valor, 1);
}
