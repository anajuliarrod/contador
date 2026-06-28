use anchor_lang::prelude::*;

// Esse ID é um placeholder. Depois do `anchor build`, rode `anchor keys sync`
// pra ele virar o ID real do SEU programa (gerado em target/deploy/contador-keypair.json).
declare_id!("J6VdM3QeQFL3m2Shh3PXA6DcmELLhQfUwCqbhykzAhSa");

#[program]
pub mod contador {
    use super::*;

    /// Cria a conta de estado e grava o valor inicial.
    /// (No modelo on-chain: o estado nasce numa conta, não no programa.)
    pub fn initialize(ctx: Context<Initialize>, valor: u64) -> Result<()> {
        let contador = &mut ctx.accounts.contador;
        contador.valor = valor;
        contador.dono = ctx.accounts.usuario.key();
        msg!("Contador inicializado com valor = {}", valor);
        Ok(())
    }

    /// Lê o valor da conta, soma +1, escreve de volta.
    /// checked_add evita overflow silencioso (footgun de aritmética).
    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let contador = &mut ctx.accounts.contador;
        contador.valor = contador
            .valor
            .checked_add(1)
            .ok_or(ContadorError::Overflow)?;
        msg!("Novo valor = {}", contador.valor);
        Ok(())
    }
}

// ---- Parte 3: structs de contas (o "ouro do Anchor") ----

#[derive(Accounts)]
pub struct Initialize<'info> {
    // init cria a conta · payer paga o rent · space = 8 (discriminador) + 8 (u64) + 32 (Pubkey)
    #[account(
        init,
        payer = usuario,
        space = 8 + 8 + 32,
    )]
    pub contador: Account<'info, Contador>,

    // mut porque o saldo do usuário muda (ele paga o rent) · Signer porque ele autoriza
    #[account(mut)]
    pub usuario: Signer<'info>,

    // necessário pra qualquer init (é o System Program que cria contas)
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    // mut porque vamos escrever · has_one = dono garante que só o dono incrementa
    #[account(mut, has_one = dono)]
    pub contador: Account<'info, Contador>,

    // o dono assina a transação de increment
    pub dono: Signer<'info>,
}

// ---- Parte 2: conta de estado ----

#[account]
pub struct Contador {
    pub valor: u64,   // 8 bytes
    pub dono: Pubkey, // 32 bytes
}

// ---- Erros customizados ----

#[error_code]
pub enum ContadorError {
    #[msg("O contador estourou o limite de u64.")]
    Overflow,
}
