# R3 Session Keys Home assignment

## How it works

The admin creates a smart wallet (`UserSmartWallet`) for each user. A session executor then creates a `Session` so it can call one target program for that wallet. The session lists which instructions are allowed and when it expires.

The session cannot be used until the smart-wallet owner approves it. The owner can revoke it at any time.

The session key is a short-lived key. The session executor creates it when it creates the session. The program uses this key to identify the session.

On `execute_with_session`, the program checks the session: it must be approved and not expired, the target program and instruction must be allowed, and remaining accounts must pass the safety rules. Then it calls the target instruction via CPI. The smart wallet PDA signs that call (`invoke_signed`).

```text
 Admin            Session executor       Smart wallet owner     R3 session-keys          Target
   |                  (bot)                   (user)                program               program
   |                     |                       |                     |                     |
   | create_smart_wallet |                       |                     |                     |
   |------------------------------------------------------------------>| creates             |
   |                     |                       |                     | UserSmartWallet PDA |
   |                     |                       |                     |                     |
   |                     | create_session        |                     |                     |
   |                     |-------------------------------------------->| Session PDA:        |
   |                     |                       |                     | WaitingForApproval  |
   |                     |                       |                     |                     |
   |                     |                       | approve_session     |                     |
   |                     |                       |-------------------->| Session PDA:        |
   |                     |                       |                     | Approved            |
   |                     |                       |                     |                     |
   |                     | execute_with_session  |                     |                     |
   |                     |                       |                     |                     |
   |                     |-------------------------------------------->| validates session   |
   |                     |                       |                     |---invoke_signed---->| executes
   |                     |                       |                     |                     | 
   |                     |                       |                     |                     | 
   |                     |                       |                     |                     |
   |                     |                       | revoke_session      |                     |
   |                     |                       |-------------------->|                     |
   |                     |                       |                     |                     |
```

## Build and test

This project uses [just](https://just.systems/) as the task runner. Install it with Homebrew, Cargo.

```console
brew install just
or
cargo install just
```

You also need [Anchor](https://www.anchor-lang.com/) 1.1.2, [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) 3.1.x.
Yarn and [Surfpool](https://surfpool.run) for running the TypeScript tests.


Build the program:

```console
just build
```

Default tests are Rust integration tests that run in-process against [LiteSVM](https://github.com/LiteSVM/litesvm):

```console
just rust
```

TypeScript Mocha tests run against Surfpool. `just ts` starts Surfpool, runs the tests, and stops it. To inspect transactions, start Surfpool in one terminal and run the tests in another:

```console
just surfpool
```

```console
just ts-surfpool
```

### Mock program

Session-key tests treat an unaware target program as the CPI destination. That mock lives in a separate repository: [gwalen/r3-mock-program](https://github.com/gwalen/r3-mock-program). It is a small Anchor program (counter plus a simple token pool).

This repo has the compiled binary and IDL copied after building the mock program.
- `tests/fixtures/mock-program/mock_program.so`
- `idls/mock_program.json`

## Framework choice
This program uses Anchor version 1.1.2, the latest stable Anchor release at the time of writing. Anchor is the most widely adopted Solana program framework and 
has an established ecosystem, documentation, tooling, and developer community.
It abstracts low-level operations such as account validation, serialization, PDA constraints, CPI construction, IDL generation, and client generation,
allowing development to focus on business logic. 

Framework selection should also consider long-term maintenance. A widely adopted framework makes it easier for other developers to understand,
review, and contribute to the program. Selecting a niche framework may introduce risks such as limited documentation, a smaller contributor pool, unstable APIs, or eventual discontinuation.

I have considered the following alternatives:

- This framework is gaining popularity and uses zero-copy, no_std, and other optimizations that significantly reduce program binary size and CU usage. Pinocchio and P-Token were audited.
Pinocchio is also a low-level framework. It requires more boilerplate and sometimes uses Rust unsafe, so it has a much higher entry level than Anchor.
Moving to Pinocchio could be considered as an optimization after the initial phase of testing with partners, if reducing binary size and CU usage becomes a priority.

- Steel: This is another low-level framework that can reduce binary size and CU usage. 
  However, it has a smaller maintainer and developer community,remains unaudited, and is still a relatively niche project.

- Anchor V2: This is the next version of Anchor, based on Pinocchio. It provides significant reductions in binary size and CU usage while still abstracting low-level operations similarly to Anchor V1.
  At the time of writing, it is not yet ready for production because it has only recently completed an audit and reached RC-1 version.
  It could become the best choice once a stable version is released.

- Quasar: This is an Anchor V2 competitor developed by Blueshift. 
  It is still in beta and unaudited, so it is not yet production-ready. A stable release may make it worth reconsidering later.  

## Potential future improvements


1. Selective use of smart-wallet funds: Allow sessions to use only explicitly approved token mints, with per-transaction and cumulative spending limits. 

2. Multiple target programs: Allow each session to use multiple target programs, each with its own list of allowed discriminators. This can be represented as a PDA-based map using the seeds [b"allowed_target", smart_wallet, session_key, target_program].

3. Program specific policy modules: Add dedicated validation modules for sensitive target programs. For example, an SPL Token policy should normally reject Approve and ApproveChecked, because they grant delegate authority over a smart wallet’s token account.

4. Account allowlists: Allow sessions to specify which accounts may be passed to an instruction and with which permissions (writability, is_signer). This would provide finer-grained control but would require more complex logic.

5. Closing session accounts: Add an instruction to close expired or revoked session accounts and return their rent to the user.

## Alternative solution
Alternatively, the target program could be aware of the session-key system and use its validation macro or helper directly. This approach was developed by Gum and is now maintained by [MagicBlock](https://github.com/magicblock-labs/magicblock-engine-examples/tree/main/session-keys/anchor).
However, it is not generic because every target program must be updated to support session keys. 
My solution keeps the target program unaware of sessions and better demonstrates Solana native account abstraction through a smart-wallet PDA.


