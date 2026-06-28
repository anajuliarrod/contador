/**
 * Interaction script — calls initialize() then increment() on the deployed
 * `contador` program (devnet) and prints the on-chain state after each step.
 *
 * Run with (provider is read from these two env vars):
 *   ANCHOR_PROVIDER_URL="<devnet RPC>" \
 *   ANCHOR_WALLET="$HOME/.config/solana/id.json" \
 *   npx ts-node app/interact.ts
 */
import * as anchor from "@anchor-lang/core";

// The IDL carries the program address + the full instruction/account layout.
// eslint-disable-next-line @typescript-eslint/no-var-requires
const idl = require("../target/idl/contador.json");

async function main() {
  // Provider = RPC connection + wallet (from ANCHOR_PROVIDER_URL / ANCHOR_WALLET).
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Because the IDL has `address`, the Program knows which program to call.
  const program = new anchor.Program(idl as anchor.Idl, provider);

  const usuario = provider.wallet.publicKey; // payer for initialize, dono for increment
  console.log("program :", program.programId.toBase58());
  console.log("usuario :", usuario.toBase58());

  // The counter's state lives in its OWN account (it's NOT a PDA here), so we
  // generate a fresh keypair — that account signs its own creation in initialize.
  const contadorKp = anchor.web3.Keypair.generate();
  console.log("contador:", contadorKp.publicKey.toBase58());

  // ---------- initialize(valor = 0) ----------
  const sigInit = await program.methods
    .initialize(new anchor.BN(0))
    .accountsPartial({
      contador: contadorKp.publicKey,
      usuario,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([contadorKp]) // new account signs its own creation
    .rpc();
  console.log("\ninitialize tx:", sigInit);

  let estado = await program.account.contador.fetch(contadorKp.publicKey);
  console.log(
    "  -> valor =",
    estado.valor.toString(),
    "| dono =",
    estado.dono.toBase58()
  );

  // ---------- increment() ----------
  const sigInc = await program.methods
    .increment()
    .accountsPartial({
      contador: contadorKp.publicKey,
      dono: usuario,
    })
    .rpc();
  console.log("\nincrement tx:", sigInc);

  estado = await program.account.contador.fetch(contadorKp.publicKey);
  console.log("  -> valor =", estado.valor.toString());

  const base = "https://explorer.solana.com";
  console.log("\n✅ initialize -> increment done on devnet");
  console.log(
    `account : ${base}/address/${contadorKp.publicKey.toBase58()}?cluster=devnet`
  );
  console.log(`init tx : ${base}/tx/${sigInit}?cluster=devnet`);
  console.log(`inc  tx : ${base}/tx/${sigInc}?cluster=devnet`);
}

main().then(
  () => process.exit(0),
  (err) => {
    console.error(err);
    process.exit(1);
  }
);
