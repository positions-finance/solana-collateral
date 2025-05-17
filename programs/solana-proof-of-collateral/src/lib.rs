use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_proof_of_collateral {
    use super::*;

    // Initialize the bridge program
    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> Result<()> {
        let bridge_state = &mut ctx.accounts.bridge_state;
        bridge_state.authority = ctx.accounts.authority.key();
        bridge_state.relayer_count = 0;
        bridge_state.bump = bump;

        msg!(
            "Bridge initialized with authority: {}",
            bridge_state.authority
        );
        Ok(())
    }

    pub fn register_token(ctx: Context<RegisterToken>, token_id: u64) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        token_info.mint = ctx.accounts.mint.key();
        token_info.token_id = token_id;
        token_info.authority = ctx.accounts.authority.key();
        token_info.is_active = true;

        msg!(
            "Token registered: ID={}, Mint={}",
            token_id,
            token_info.mint
        );
        Ok(())
    }

    pub fn add_relayer(ctx: Context<AddRelayer>, relayer_address: Pubkey) -> Result<()> {
        let bridge_state = &mut ctx.accounts.bridge_state;
        let relayer_info = &mut ctx.accounts.relayer_info;

        require_keys_eq!(
            ctx.accounts.authority.key(),
            bridge_state.authority,
            BridgeError::UnauthorizedAccess
        );

        relayer_info.relayer = relayer_address;
        relayer_info.authority = ctx.accounts.authority.key();
        relayer_info.is_active = true;
        relayer_info.index = bridge_state.relayer_count;

        bridge_state.relayer_count += 1;

        msg!("Added relayer: {}", relayer_address);
        Ok(())
    }

    pub fn remove_relayer(ctx: Context<RemoveRelayer>) -> Result<()> {
        let bridge_state = &ctx.accounts.bridge_state;
        let relayer_info = &mut ctx.accounts.relayer_info;

        require_keys_eq!(
            ctx.accounts.authority.key(),
            bridge_state.authority,
            BridgeError::UnauthorizedAccess
        );

        relayer_info.is_active = false;

        msg!("Removed relayer: {}", relayer_info.relayer);
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, deposit_id: u64, amount: u64) -> Result<()> {
        let token_info = &ctx.accounts.token_info;

        require!(token_info.is_active, BridgeError::TokenNotActive);

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.bridge_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)?;

        let deposit_info = &mut ctx.accounts.deposit_info;
        deposit_info.deposit_id = deposit_id;
        deposit_info.token_id = token_info.token_id;
        deposit_info.mint = token_info.mint;
        deposit_info.amount = amount;
        deposit_info.depositor = ctx.accounts.user.key();
        deposit_info.claimed = false;
        deposit_info.timestamp = Clock::get()?.unix_timestamp;

        emit!(DepositEvent {
            deposit_id,
            token_id: token_info.token_id,
            amount,
            depositor: ctx.accounts.user.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!(
            "Deposit successful: ID={}, Amount={}, Token={}",
            deposit_id,
            amount,
            token_info.mint
        );
        Ok(())
    }

    pub fn request_withdrawal(
        ctx: Context<RequestWithdrawal>,
        deposit_id: u64,
        recipient: Pubkey,
    ) -> Result<()> {
        let deposit_info = &ctx.accounts.deposit_info;
        require!(!deposit_info.claimed, BridgeError::AlreadyClaimed);

        emit!(WithdrawalRequestEvent {
            deposit_id,
            token_id: deposit_info.token_id,
            amount: deposit_info.amount,
            recipient,
            requester: ctx.accounts.requester.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!(
            "Withdrawal requested: DepositID={}, Recipient={}",
            deposit_id,
            recipient
        );
        Ok(())
    }

    pub fn process_withdrawal(
        ctx: Context<ProcessWithdrawal>,
        deposit_id: u64,
        recipient: Pubkey,
    ) -> Result<()> {
        let deposit_info = &mut ctx.accounts.deposit_info;
        let relayer_info = &ctx.accounts.relayer_info;
        let token_bridge = &ctx.accounts.token_bridge;

        require!(relayer_info.is_active, BridgeError::RelayerNotActive);

        require!(!deposit_info.claimed, BridgeError::AlreadyClaimed);

        let bridge_signer_seeds: &[&[&[u8]]] =
            &[&[b"token_bridge", &[ctx.accounts.bridge_state.bump]]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.bridge_token_account.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: token_bridge.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, bridge_signer_seeds);

        token::transfer(cpi_ctx, deposit_info.amount)?;

        deposit_info.claimed = true;

        emit!(WithdrawalCompletedEvent {
            deposit_id,
            token_id: deposit_info.token_id,
            amount: deposit_info.amount,
            recipient,
            relayer: ctx.accounts.relayer.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!(
            "Withdrawal processed: DepositID={}, Amount={}, Recipient={}, Relayer={}",
            deposit_id,
            deposit_info.amount,
            recipient,
            ctx.accounts.relayer.key()
        );
        Ok(())
    }
}

#[event]
pub struct DepositEvent {
    pub deposit_id: u64,
    pub token_id: u64,
    pub amount: u64,
    pub depositor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalRequestEvent {
    pub deposit_id: u64,
    pub token_id: u64,
    pub amount: u64,
    pub recipient: Pubkey,
    pub requester: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalCompletedEvent {
    pub deposit_id: u64,
    pub token_id: u64,
    pub amount: u64,
    pub recipient: Pubkey,
    pub relayer: Pubkey,
    pub timestamp: i64,
}

// Custom error types
#[error_code]
pub enum BridgeError {
    #[msg("Unauthorized access")]
    UnauthorizedAccess,
    #[msg("Token not registered or active")]
    TokenNotActive,
    #[msg("Deposit already claimed")]
    AlreadyClaimed,
    #[msg("Relayer not active")]
    RelayerNotActive,
}

// Initialize the bridge program with the authority
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + BridgeState::LEN,
        seeds = [b"bridge_state"],
        bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Register a token to be supported by the bridge
#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct RegisterToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump,
        constraint = bridge_state.authority == authority.key() @ BridgeError::UnauthorizedAccess
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        init,
        payer = authority,
        space = 8 + TokenInfo::LEN,
        seeds = [b"token_info", token_id.to_le_bytes().as_ref()],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,

    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
}

// Add a relayer to the system
#[derive(Accounts)]
#[instruction(relayer_address: Pubkey)]
pub struct AddRelayer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        init,
        payer = authority,
        space = 8 + RelayerInfo::LEN,
        seeds = [b"relayer_info", relayer_address.as_ref()],
        bump
    )]
    pub relayer_info: Account<'info, RelayerInfo>,

    pub system_program: Program<'info, System>,
}

// Remove a relayer from the system
#[derive(Accounts)]
pub struct RemoveRelayer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        mut,
        seeds = [b"relayer_info", relayer_info.relayer.as_ref()],
        bump
    )]
    pub relayer_info: Account<'info, RelayerInfo>,
}

// Deposit tokens with a specific deposit ID
#[derive(Accounts)]
#[instruction(deposit_id: u64, amount: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        seeds = [b"token_info", &token_info.token_id.to_le_bytes()],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,

    #[account(
        mut,
        constraint = user_token_account.mint == token_info.mint,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_bridge"],
        bump = bridge_state.bump
    )]
    /// CHECK: This is the PDA that acts as the token bridge
    pub token_bridge: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = bridge_token_account.mint == token_info.mint,
        constraint = bridge_token_account.owner == token_bridge.key()
    )]
    pub bridge_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        space = 8 + DepositInfo::LEN,
        seeds = [b"deposit_info", deposit_id.to_le_bytes().as_ref()],
        bump
    )]
    pub deposit_info: Account<'info, DepositInfo>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Request a withdrawal, which emits an event for relayers
#[derive(Accounts)]
#[instruction(deposit_id: u64, recipient: Pubkey)]
pub struct RequestWithdrawal<'info> {
    #[account(mut)]
    pub requester: Signer<'info>,

    #[account(
        seeds = [b"deposit_info", &deposit_id.to_le_bytes()],
        bump,
        constraint = !deposit_info.claimed @ BridgeError::AlreadyClaimed
    )]
    pub deposit_info: Account<'info, DepositInfo>,
}

// Process a withdrawal by a relayer
#[derive(Accounts)]
#[instruction(deposit_id: u64, recipient: Pubkey)]
pub struct ProcessWithdrawal<'info> {
    #[account(mut)]
    pub relayer: Signer<'info>,

    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        seeds = [b"relayer_info", relayer.key().as_ref()],
        bump,
        constraint = relayer_info.relayer == relayer.key() @ BridgeError::UnauthorizedAccess,
        constraint = relayer_info.is_active @ BridgeError::RelayerNotActive
    )]
    pub relayer_info: Account<'info, RelayerInfo>,

    #[account(
        mut,
        seeds = [b"deposit_info", &deposit_id.to_le_bytes()],
        bump,
        constraint = !deposit_info.claimed @ BridgeError::AlreadyClaimed
    )]
    pub deposit_info: Account<'info, DepositInfo>,

    #[account(
        seeds = [b"token_bridge"],
        bump = bridge_state.bump
    )]
    /// CHECK: This is the PDA that acts as the token bridge
    pub token_bridge: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = bridge_token_account.mint == deposit_info.mint,
        constraint = bridge_token_account.owner == token_bridge.key()
    )]
    pub bridge_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = recipient_token_account.mint == deposit_info.mint,
        constraint = recipient_token_account.owner == recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// State accounts
#[account]
pub struct BridgeState {
    pub authority: Pubkey,  // Admin authority for the bridge
    pub relayer_count: u32, // Number of registered relayers
    pub bump: u8,           // Bump seed for PDA
}

#[account]
pub struct TokenInfo {
    pub mint: Pubkey,      // Token mint address
    pub token_id: u64,     // Token ID in the bridge system
    pub authority: Pubkey, // Authority who registered this token
    pub is_active: bool,   // Whether this token is active
}

#[account]
pub struct RelayerInfo {
    pub relayer: Pubkey,   // Relayer public key
    pub authority: Pubkey, // Authority who added this relayer
    pub is_active: bool,   // Whether this relayer is active
    pub index: u32,        // Index in the relayer list
}

#[account]
pub struct DepositInfo {
    pub deposit_id: u64,   // Unique ID for this deposit
    pub token_id: u64,     // Token ID
    pub mint: Pubkey,      // Token mint address
    pub amount: u64,       // Amount of tokens deposited
    pub depositor: Pubkey, // Account that deposited the tokens
    pub claimed: bool,     // Whether this deposit has been claimed
    pub timestamp: i64,    // Timestamp of the deposit
}

impl BridgeState {
    pub const LEN: usize = 32 + 4 + 1; // Pubkey + u32 + u8
}

impl TokenInfo {
    pub const LEN: usize = 32 + 8 + 32 + 1; // Pubkey + u64 + Pubkey + bool
}

impl RelayerInfo {
    pub const LEN: usize = 32 + 32 + 1 + 4; // Pubkey + Pubkey + bool + u32
}

impl DepositInfo {
    pub const LEN: usize = 8 + 8 + 32 + 8 + 32 + 1 + 8; // u64 + u64 + Pubkey + u64 + Pubkey + bool + i64
}
