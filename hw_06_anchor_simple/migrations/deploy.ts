// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.
// 初始化链上数据相关逻辑脚本

const { web3, workspace, BN, setProvider } = require("@project-serum/anchor");
const { TokenInstructions } = require('@project-serum/serum');
const { createMint } = require("@project-serum/common")

const createToken = async (provider, program, tokenConfig) => {
  const tokenOwnerAccount = web3.Keypair.generate();

  const [tokenAuthority, tokenNonce] = await web3.PublicKey.findProgramAddress(
      [tokenOwnerAccount.publicKey.toBuffer()],
      program.programId
  );

  const splToken = await createMint(
      provider,
      tokenAuthority,
      tokenConfig.decimals
  );

  console.log(`Created ${tokenConfig.symbol} Token`, splToken.toBase58());

  return {
    tokenOwnerAccount,
    splToken,
    tokenNonce,
    tokenAuthority,
  };
}

module.exports = async function (provider) {
  setProvider(provider);

  const faucetProgram = workspace.Faucet;
  const wallet = provider.wallet;

  const tokenConfigs = [
    {
      symbol: 'btc',
      name: 'Wrapped Bitcoin',
      decimals: 8,
      dripVolume: new BN(10 ** 8)
    },
    {
      symbol: 'eth',
      name: 'Wrapped Ether',
      decimals: 8,
      dripVolume: new BN(10 ** 8)
    }
  ];

  for (const tokenConfig of tokenConfigs) {
    const { tokenOwnerAccount: faucetConfigAccount, splToken, tokenNonce, tokenAuthority } = await createToken(provider, faucetProgram, tokenConfig);

    console.log(tokenConfig.symbol, "faucet_config address: ", faucetConfigAccount.publicKey.toBase58());

    await faucetProgram.rpc.initialize(tokenNonce, tokenConfig.dripVolume, {
      accounts: {
        faucetConfig: faucetConfigAccount.publicKey,
        tokenMint: splToken,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        tokenAuthority,
        rent: web3.SYSVAR_RENT_PUBKEY
      },
      signers: [faucetConfigAccount],
      instructions: [
        await faucetProgram.account.faucetConfig.createInstruction(faucetConfigAccount)
      ],
    });
  }
}
