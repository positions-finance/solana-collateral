# Solana Proof of Collateral

A powerful Solana-based protocol that enables cross-chain collateral proofs with secure token bridging capabilities, allowing locked assets to be utilized across multiple blockchain ecosystems.

## Overview

This project provides a robust and secure bridge infrastructure for depositing, verifying, and withdrawing tokens across different blockchain networks. It empowers users to lock tokens on Solana and generate cryptographically verifiable proof of collateral, which can then be utilized on other chains to access DeFi services without moving the underlying assets.

Key features:
- Secure token registration and management system
- On-chain verification for token deposits with tamper-proof record keeping
- Cross-chain withdrawal requests with multi-layer security checks
- Authorized relayer network for trustless bridging operations
- Comprehensive event tracking and logging for complete transparency
- Gas-efficient implementations for cost-effective operations

## Integration with Positions Finance

This protocol powers the cross-chain collateral verification system for [Positions Finance](https://positions.finance/), a DeFi platform that allows users to unlock the value of their locked assets.

Positions Finance leverages this Solana Proof of Collateral system to:
- Verify staked/locked assets across multiple ecosystems in a trustless environment
- Enable users to borrow against their locked tokens without unstaking
- Provide credit liquidity based on verifiable on-chain collateral
- Support staking, earning, borrowing, and leveraging functionalities
- Create a seamless cross-chain DeFi experience

The protocol's events and proofs are utilized by Positions Finance to maintain an accurate, real-time record of collateralized assets, enabling safe lending against these positions.

## Architecture

The system is built on three main components that work together to ensure security and efficiency:

1. **Bridge State** - Central registry that maintains the bridge authority, security parameters, and relayer network
   - Manages system-wide configuration and security policies
   - Tracks authorized relayers and their performance metrics
   - Implements governance controls for protocol updates

2. **Token Registry** - Comprehensive management system for supported tokens
   - Maintains token mappings across different chains
   - Enforces token-specific validation rules and limits
   - Provides flexibility for adding new token types

3. **Deposit/Withdrawal System** - Advanced handling of token movements and proof generation
   - Implements atomic operations for deposit confirmation
   - Generates cryptographic proofs for cross-chain verification
   - Enforces time-locked security measures for withdrawals
   - Maintains comprehensive audit trail of all operations

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or later)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.14.0 or later)
- [Yarn](https://yarnpkg.com/getting-started/install) (v1.22.0 or later)
- [Node.js](https://nodejs.org/en/download/) (v16.0.0 or later)
- [Anchor](https://project-serum.github.io/anchor/getting-started/installation.html) (v0.31.0)

## Installation

1. Clone the repository:
   ```bash
   git clone <your-repo-url>
   cd solana-collateral
   ```

2. Install dependencies:
   ```bash
   yarn install
   ```

3. Build the program:
   ```bash
   anchor build
   ```

## Configuration

1. Update the program ID in `Anchor.toml` and `lib.rs` with your own program ID:
   ```bash
   solana-keygen new -o target/deploy/solana_proof_of_collateral-keypair.json
   anchor keys sync
   ```

2. Update the `Anchor.toml` with appropriate settings for deployment:
   - Set the desired Solana cluster (localnet, devnet, or mainnet)
   - Configure appropriate RPC endpoints
   - Adjust memory settings for optimal performance

## Technical Implementation Details

The protocol implements several advanced mechanisms to ensure security and reliability:

- **PDA (Program Derived Address)** architecture for secure token custody
- **Seed-based derivation** for deterministic account generation
- **Event-driven architecture** for efficient cross-chain communication
- **Atomic transactions** to prevent partial execution vulnerabilities
- **Relayer consensus mechanism** to prevent malicious behavior

## Usage

### Local Development

1. Start a local Solana validator:
   ```bash
   solana-test-validator
   ```

2. Run the tests:
   ```bash
   anchor test
   ```

3. For detailed logging during development:
   ```bash
   RUST_LOG=debug anchor test
   ```

### Deployment

1. Deploy to localnet:
   ```bash
   anchor deploy
   ```

2. For devnet or mainnet deployment, update the `cluster` in `Anchor.toml` and run:
   ```bash
   anchor deploy --provider.cluster devnet
   ```

3. For production deployments, use a hardware wallet:
   ```bash
   anchor deploy --provider.wallet /path/to/keypair.json
   ```

## Smart Contract Functions

### Admin Functions

- `initialize` - Set up the bridge with an authority address and security parameters
- `register_token` - Add a token to the bridge registry with customizable parameters
- `add_relayer` - Add an authorized relayer for cross-chain operations with specific permissions
- `remove_relayer` - Deactivate a relayer from the authorized network

### User Functions

- `deposit` - Lock tokens in the bridge and generate cryptographic proof of collateral
- `request_withdrawal` - Request to withdraw tokens to a specified recipient with validation checks
- `process_withdrawal` - Process a withdrawal request with multi-signature verification (relayer only)

### Advanced Features

- **Time-locked withdrawals** - Security feature to prevent flash loan attacks
- **Multi-signature validation** - For high-value transactions
- **Circuit breaker mechanism** - Automatic pause in case of suspicious activity
- **Fee management system** - Configurable fee structure for sustainability

## Security Considerations

- The bridge authority has administrative control over token registration and relayer management
- All token transfers are validated against respective token mints with multiple verification layers
- Deposits can only be claimed once to prevent double-spending attacks
- Only active, authorized relayers can process withdrawals
- Circuit breakers can pause the system in case of detected exploits
- Regular security audits are recommended before production deployment

## Integration Guide

To integrate this protocol with your own applications:

1. Initialize a connection to the program using Anchor:
   ```typescript
   const program = new Program<SolanaProofOfCollateral>(
     IDL,
     PROGRAM_ID,
     provider
   );
   ```

2. Create user deposit functions:
   ```typescript
   async function depositTokens(amount, tokenId) {
     // Implementation details...
   }
   ```

3. Listen for deposit events to trigger cross-chain actions:
   ```typescript
   program.addEventListener('DepositEvent', (event) => {
     // Handle the deposit event
   });
   ```

## Development

For local development and testing:

1. Set up a local validator: `solana-test-validator`
2. Build the program: `anchor build`
3. Deploy locally: `anchor deploy`
4. Extend the test script in `tests/solana-proof-of-collateral.ts` to cover your use cases
5. Simulate different network conditions to ensure robustness

## License

ISC

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

To contribute:
1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## Resources

- [Positions Finance](https://positions.finance/) - Main platform leveraging this protocol
- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework](https://www.anchor-lang.com/) 