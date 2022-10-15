// 测试相关逻辑

import { BN, getProvider, web3, workspace } from "@project-serum/anchor";
// 注意：@project-serum/common库以过期无法使用，会抱函数不存在的错误
import {
  createMint,
  createTokenAccountInstrs,
  getMintInfo,
  getTokenAccount,
} from "@project-serum/common";
import { TokenInstructions } from "@project-serum/serum";
import assert from "assert";
import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
const { SystemProgram } = anchor.web3;
import { PublicKey } from '@solana/web3.js';
const { TOKEN_PROGRAM_ID, Token, ASSOCIATED_TOKEN_PROGRAM_ID } = require("@solana/spl-token");
import { Hw06AnchorSimple } from '../target/types/hw_06_anchor_simple';


describe("Hw06AnchorSimple", () => {
  console.log("start...");
  //const provider = anchor.Provider.env();
  // @ts-ignore
  const provider = new anchor.getProvider();
  anchor.setProvider(provider);
  const faucetProgram = workspace.Hw06AnchorSimple as Program<Hw06AnchorSimple>;

  let faucetConfig: web3.Keypair;
  let testTokenMint: web3.PublicKey;
  let testTokenAuthority: web3.PublicKey;
  let nonce: number;

  const testTokenDecimals = 9;
  const dripVolume: BN = new BN(10 ** testTokenDecimals);
  const dripVolume_next: BN = new BN(10 ** testTokenDecimals + 1);

  // 测试开始先做一些初始化的操作（注意：由于@project-serum/common库以过期无法使用，会抱函数不存在的错误）
  before(async () => {
    faucetConfig = web3.Keypair.generate();
    [testTokenAuthority, nonce] = await web3.PublicKey.findProgramAddress(
        [faucetConfig.publicKey.toBuffer()],
        faucetProgram.programId
    );
    console.log("createMint...");
    testTokenMint = await createMint(provider, testTokenAuthority, testTokenDecimals);
    console.log("faucetConfig:", faucetConfig.publicKey.toString());
    console.log("faucetProgram.programId", faucetProgram.programId.toString());
    console.log("testTokenAuthority:", testTokenAuthority.toString());
    console.log("nonce", nonce);
    console.log("testTokenMint", testTokenMint.toString());
  });

  // 测试合约initialize函数（注意：由于@project-serum/common库以过期无法使用，会抱函数不存在的错误）
  describe("# initialize", () => {
    it("Should init successful", async () => {
      await faucetProgram.rpc.initialize(nonce, dripVolume, {
        accounts: {
          faucetConfig: faucetConfig.publicKey,
          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
          tokenMint: testTokenMint,
          tokenAuthority: testTokenAuthority,
          rent: web3.SYSVAR_RENT_PUBKEY,

          user: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId
        },
        signers: [faucetConfig],
      });

      const faucetConfigAccount = await faucetProgram.account.faucetConfig.fetch(faucetConfig.publicKey);

      assert.strictEqual(
          faucetConfigAccount.tokenProgram.toBase58(),
          TokenInstructions.TOKEN_PROGRAM_ID.toBase58()
      );
      assert.strictEqual(
          faucetConfigAccount.tokenMint.toBase58(),
          testTokenMint.toBase58()
      );
      assert.strictEqual(
          faucetConfigAccount.tokenAuthority.toBase58(),
          testTokenAuthority.toBase58()
      );
      assert.strictEqual(faucetConfigAccount.nonce, nonce);
      assert.strictEqual(
          faucetConfigAccount.dripVolume.toNumber(),
          dripVolume.toNumber()
      );
    });
    it("Updates Drip Volume", async () => {
      await faucetProgram.rpc.setDripVolume(dripVolume_next, {
        accounts: {
          faucetConfig: faucetConfig.publicKey,
          authority: provider.wallet.publicKey,
        },
      });

      const configAccount = await faucetProgram.account.faucetConfig.fetch(faucetConfig.publicKey);

      assert.ok(configAccount.authority.equals(provider.wallet.publicKey));
      assert.ok(configAccount.dripVolume.toNumber() == dripVolume_next.toNumber());
    });
  });

  // 测试合约drip函数（注意：由于@project-serum/common库以过期无法使用，会抱函数不存在的错误）
  describe("# drip", () => {
    it("Should drip successful", async () => {
      const signers: web3.Keypair[] = [];
      const instructions: web3.TransactionInstruction[] = [];
      const receiver = web3.Keypair.generate();
      const receiverTokenAccount = web3.Keypair.generate();
      instructions.push(
          ...(await createTokenAccountInstrs(
              provider,
              receiverTokenAccount.publicKey,
              testTokenMint,
              receiver.publicKey
          ))
      );
      signers.push(receiverTokenAccount);

      const tokenMintInfo = await getMintInfo(provider, testTokenMint);
      await faucetProgram.rpc.drip({
        accounts: {
          faucetConfig: faucetConfig.publicKey,
          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
          tokenMint: testTokenMint,
          receiver: receiverTokenAccount.publicKey,
          tokenAuthority: tokenMintInfo.mintAuthority!!
        },
        instructions: instructions,
        signers: signers,
      });

      const tokenAccount = await getTokenAccount(
          provider,
          receiverTokenAccount.publicKey
      );

      assert.strictEqual(tokenAccount.amount.toNumber(), dripVolume_next.toNumber());
    });
  });
});
