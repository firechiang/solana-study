/**
 * @flow
 */

import {
    PublicKey,
    SystemProgram,
    TransactionInstruction,
    SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js"
import * as BufferLayout from '@solana/buffer-layout'

/**
 * HelloWorld
 */
export class HelloWorld {
    static createHelloInstruction(
        playerAccountKey,
        messageAccountKey,
        programID,
        message,
    ) {

        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("i"),
            BufferLayout.blob(message.length,"message"),
        ]);

        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode(
            {
              i:0, // hello
              message: Buffer.from(message, 'utf8'),
            },
            data,
        );

        let keys = [
            // is_signer 表示访问者是否持有私钥，is_writable 表示合约程序是否可以修改账户信息
            {pubkey: playerAccountKey, isSigner: true, isWritable: true},
            {pubkey: messageAccountKey, isSigner: true, isWritable: true},
        ];

        const  trxi = new TransactionInstruction({
            keys,
            programId: programID,
            data,
        });
        return trxi;
    }


    static createEraseInstruction(
        playerAccountKey,
        messageAccountKey,
        programID,
        message,
    ) {

        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("i"),
        ]);

        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode(
            {
              i:1, // erase
            },
            data,
        );

        let keys = [
            {pubkey: playerAccountKey, isSigner: true, isWritable: true},
            {pubkey: messageAccountKey, isSigner: true, isWritable: true},
        ];

        const  trxi = new TransactionInstruction({
            keys,
            programId: programID,
            data,
        });
        return trxi;
    }

    static createQueryInstruction(
        playerAccountKey,
        messageAccountKey,
        programID,
        message,
    ) {

        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("i"),
        ]);

        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode(
            {
                i:2, // erase
            },
            data,
        );

        let keys = [
            {pubkey: playerAccountKey, isSigner: true, isWritable: true},
            {pubkey: messageAccountKey, isSigner: true, isWritable: true},
        ];

        const  trxi = new TransactionInstruction({
            keys,
            programId: programID,
            data,
        });
        return trxi;
    }
}