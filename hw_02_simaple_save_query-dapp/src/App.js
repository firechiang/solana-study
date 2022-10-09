import React from 'react';
import logo from './logo.svg';
import './App.css';
import TextField from '@material-ui/core/TextField'
import Container from '@material-ui/core/Box'
import Divider from '@material-ui/core/Divider'
import * as Layout from './Layout'

import { LAMPORTS_PER_SOL,Account, PublicKey, Connection, SystemProgram ,Transaction,sendAndConfirmTransaction} from '@solana/web3.js';
import { Button,Grid } from '@material-ui/core';
import {HelloWorld} from './HelloWorld';



class Content extends React.Component {

  constructor(props) {
    super(props)
    this.state = { };
    this.onErase = this.onErase.bind(this);
    this.onHello = this.onHello.bind(this);
    this.onQuery = this.onQuery.bind(this);

    //let url =  'http://api.mainnet-beta.solana.com';
    //let url =  'https://solana-api.projectserum.com';
    //let url =  'https://api.devnet.solana.com';
    let url = "http://127.0.0.1:8899";
    this.connection = new Connection(url);
    // 8B7QfHs8gFKmSLfTSiLoHCDntZ94BKoaEVe82sxBqjei
    this.programID = new PublicKey("8gSU73AzuwE2SgVyU62cTgBF2YPqoKLr7hnhxZf31Go7");
    this.messageAccount = new Account();
    this.playerPrivKey = [82,225,254,52,201,68,118,133,116,22,32,184,203,37,211,93,152,39,183,144,80,221,96,180,32,218,118,10,16,137,239,56,223,133,86,36,186,139,88,155,30,10,104,80,106,84,55,105,47,30,248,38,224,88,195,126,248,29,120,173,150,207,110,168];
    this.playerAccount = new Account(this.playerPrivKey);
  }


  render() {
    return (
        <Container>


          <React.Fragment>
            <Button onClick={this.onHello}> 存储数据</Button>
          </React.Fragment>
          <Divider />
          <React.Fragment>
            <Button onClick={this.onErase}> 删除数据</Button>
          </React.Fragment>
          <Divider />
          <React.Fragment>
            <Button onClick={this.onQuery}> 查询数据</Button>
          </React.Fragment>
        </Container>
    );
  }

  async onQuery() {
      let trxi = HelloWorld.createQueryInstruction(
          this.playerAccount.publicKey,
          this.messageAccount.publicKey,
          this.programID,
      );

      const transaction = new Transaction();
      transaction.add(trxi);

      let signers= [this.playerAccount, this.messageAccount];
      sendAndConfirmTransaction(this.connection, transaction, signers, {
          skipPreflight: false,
          commitment: 'recent',
          preflightCommitment: 'recent',
      }).then((data)=>{
          console.log(data);
      }).catch((e)=>{
          console.log("error:", e);
      })

  }

  async onErase() {
    let trxi = HelloWorld.createEraseInstruction(
        this.playerAccount.publicKey,
        this.messageAccount.publicKey,
        this.programID,
    );

    const transaction = new Transaction();
    transaction.add(trxi);

    let signers= [this.playerAccount, this.messageAccount];
    sendAndConfirmTransaction(this.connection, transaction, signers, {
      skipPreflight: false,
      commitment: 'recent',
      preflightCommitment: 'recent',
    }).then(()=>{
      console.log("done erase");
    }).catch((e)=>{
      console.log("error:", e);
    })
  }

  async onHello() {
      // 存储指定长度的数据所需要的价格（注意：这个钱由发起账户会自动打到数据账户）
      let messageNeeded = await this.connection.getMinimumBalanceForRentExemption(Layout.messagSpace);
      console.info("账户数据长度=",messageNeeded);
      // 创建存储数据账户（因为要存储数据所以我们需要指定数据最大长度）
      const trxi0 =  SystemProgram.createAccount({
          fromPubkey: this.playerAccount.publicKey,
          newAccountPubkey: this.messageAccount.publicKey,
          lamports: messageNeeded,
          space: Layout.messagSpace,
          programId: this.programID,
      });

      console.log("playerAccount:", this.playerAccount.publicKey.toBase58());
      console.log("message:", this.messageAccount.publicKey.toBase58());

      // 创建调用智能合约参数也就是生成 input数据
      let trxi = HelloWorld.createHelloInstruction(
          this.playerAccount.publicKey,
          this.messageAccount.publicKey,
          this.programID,
          "hello world!ssdfsdfsdfsd",
      );

      const transaction = new Transaction();
      transaction.add(trxi0);
      transaction.add(trxi);

      let signers= [this.playerAccount, this.messageAccount];
      sendAndConfirmTransaction(this.connection, transaction, signers, {
          skipPreflight: false,
          commitment: 'recent',
          preflightCommitment: 'recent',
      }).then(()=>{
          console.log("done hello");
      }).catch((e)=>{
          console.log("error:", e);
      })
  }
}


function App() {
  return (
      <Content />
  );
}

export default App;