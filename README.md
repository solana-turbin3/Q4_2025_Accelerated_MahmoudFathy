
# Turbin3 Accelerated Builder Cohort [Q4]

Work during the Q4 Accelerated Builder Cohort program

## Token Extension

[dir - `tokenextension-whitelist`]

- Challenge by Andre: Implement a hashmap-like whitelist instead of the applied whitelist vector. 
- This is more efficient than the whitelist vector. 

**Walkthrough**

- Function `add_to_whitelist` now creates new account (reference to source token account), hence I apply this change:

```rust
    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(ctx.bumps)
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(ctx.bumps)
    }
```
- `user` refers to the owner of the source token account.
- This is passed to `whitelist_operations` :

```rust
#[instruction(user: Pubkey)]
pub struct WhitelistOperations<'info> {
    #[account(
        mut,
        //address = 
    )]
    pub admin: Signer<'info>,
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"whitelist", user.key().as_ref()],
        space = 8 + Whitelist::INIT_SPACE,
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}
```
- `user` is passed as an argument to be used as seed for `whitelist`, this is a change to `whitelist` account which used to be providing a record for the vector of all whitelisted addresses. Now `whitelist` is just a pda the provides a flag for one given user if they are whitelisted or not, shown here: 

```rust
#[account]
#[derive(InitSpace)]
pub struct Whitelist {
    pub is_whitelisted: bool,
    pub bump: u8,
}
```

- The important part is to let `ExtraAccountMetaList` be aware about the account it should pass on: 

```rust
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        Ok(
            vec![
                ExtraAccountMeta::new_with_seeds(
                    &[
                        Seed::Literal {
                            bytes: b"whitelist".to_vec(),
                        },
                        Seed::AccountKey { index: 0 }  // Should refer to user pubkey
                    ],
                    false, // is_signer
                    false // is_writable
                )?
            ]
        )
    }
```

- Remove any reference to `initialize_white_list` since it is not needed anymore.

- Finally, test is updated accordingly: 

```typescript

  it("Add user to whitelist", async () => {
      const whitelist = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("whitelist"),
          provider.publicKey.toBuffer()
        ],
        program.programId
      )[0];

      const tx = await program.methods.addToWhitelist(provider.publicKey)
          // WhitelistOperations
          .accountsPartial({
            admin: provider.publicKey,
            whitelist,
            systemProgram: anchor.web3.SystemProgram.programId,

        })
        .rpc();

      console.log("\nUser added to whitelist:", provider.publicKey.toBase58());
      console.log("Transaction signature:", tx);
  });

```



## LiteSVM

