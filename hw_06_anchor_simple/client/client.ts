const fs = require("fs");
const path = require("path");
const { Connection, Account } = require("@solana/web3.js");
const { Provider, Wallet, Program, web3 } = require("@project-serum/anchor");
const { createTokenAccountInstrs, getMintInfo } = require('@project-serum/common');
const { TokenInstructions } = require('@project-serum/serum');

const useWallet = (endpoint, secretKeyPath) => {
    const connection = new Connection(endpoint, "recent");
    const secretKey = fs.readFileSync(secretKeyPath);
    const payer = new Account(JSON.parse(secretKey));
    const wallet = new Wallet(payer)
    const provider = new Provider(connection, wallet, {
        commitment: 'recent'
    });

    return {
        provider,
        wallet,
    }
}

const useFaucetProgram = (provider, programId, programIdl) => {
    return new Program(
        programIdl,
        programId,
        provider
    )
}

const getTokenAccount = async (provider, tokenMint, owner) => {
    const { value } = await provider.connection.getTokenAccountsByOwner(
        owner,
        { mint: tokenMint },
        'recent'
    );
    return value.length ? value[0].pubkey : undefined;
}

const getAirdrop = async (provider, faucetProgram, owner, tokenMint, faucetConfig) => {
    const signers = [];
    const instructions = [];


    let receiverTokenAccountPk = await getTokenAccount(provider, tokenMint, owner);

    if (!receiverTokenAccountPk) {
        const receiverTokenAccount = new web3.Account();
        receiverTokenAccountPk = receiverTokenAccount.publicKey;
        instructions.push(
            ...(await createTokenAccountInstrs(
                provider,
                receiverTokenAccount.publicKey,
                tokenMint,
                owner
            ))
        );
        signers.push(receiverTokenAccount);
    }

    const tokenMintInfo = await getMintInfo(provider, tokenMint);

    await faucetProgram.rpc.drip({
        accounts: {
            faucetConfig,
            receiver: receiverTokenAccountPk,
            tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
            tokenMint,
            tokenAuthority: tokenMintInfo.mintAuthority
        },
        instructions: instructions.length ? instructions : undefined,
        signers: signers.length ? signers : undefined
    });

    return {
        tokenAccount: receiverTokenAccountPk
    };
}

const main = async () => {
    const secretKeyPath = path.resolve(process.env.HOME, ".config/solana/id.json");
    const endpoint = "https://api.devnet.solana.com";

    const { provider, wallet } = useWallet(endpoint, secretKeyPath);

    console.log("use wallet: ", wallet.publicKey.toBase58());

    const SOLBalance = await provider.connection.getBalance(wallet.publicKey);

    console.log("SOL Balance: ", SOLBalance.toString())

    const programId = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS";
    const programIdl = require("./hw_06_anchor_simple.json");

    const faucetProgram = useFaucetProgram(provider, programId, programIdl);

    // const BTCMint = new web3.PublicKey("ANHiz1KEuXKw3JVCdqYTFVUi2V3SXuA8TycF88XEf1QJ");
    // const BTCFaucetConfig = new web3.PublicKey("FPfjDG7beUUiEqYbWbYar3XYH2DW5sNXnAq6986ZXJuY");
    // const ETHMint = new web3.PublicKey("E8okefVR6d6RJTrdvCuPFW6Gcg9LZuPgryagUDrztpXS");
    // const ETHFaucetConfig = new web3.PublicKey("4LWsno5UJxbHannaFjsoCmHnjJvd7jREM7XQtLtMy5ZR");

    //console.log('Get BTC airdrop')

    //const { tokenAccount } = await getAirdrop(provider, faucetProgram, wallet.publicKey, BTCMint, BTCFaucetConfig);

    //const { value: newTokenBalance } = await provider.connection.getTokenAccountBalance(tokenAccount);

    //console.log('BTC balance: ', newTokenBalance.amount);

}

main();