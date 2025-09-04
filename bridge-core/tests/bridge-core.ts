import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BridgeCore } from "../target/types/bridge_core";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { Keypair, PublicKey } from "@solana/web3.js";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

describe("bridge-core", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BridgeCore as Program<BridgeCore>;
  // const programKeypair = Keypair.fromSeed(new Uint8Array([23, 249, 122, 23, 140, 230, 87, 97, 230, 92, 132, 254, 163, 126, 104, 189, 97, 78, 82, 51, 34, 154, 191, 144, 6, 239, 168, 118, 41, 178, 35, 227]));
  // console.log(programKeypair.publicKey.toBase58());
  // const feePayer = Keypair.fromSeed(new Uint8Array([204, 242, 149, 254, 47, 114, 17, 77, 92, 21, 107, 227, 160, 39, 189, 144, 168, 197, 98, 116, 172, 137, 141, 126, 58, 198, 105, 50, 59, 33, 28, 48]));
  // console.log(feePayer.publicKey.toBase58())
  let bridgeConfig: anchor.web3.PublicKey;
  let bump: number;

  it("Initializes", async () => {
    let pk = new PublicKey("9TGEJG4rSaaST7N3H8qCPZByhTXoyfvre7USxgnv6a1F");
    console.log(pk.toBase58());
    return;
    // [bridgeConfig, bump] = await anchor.web3.PublicKey.findProgramAddressSync(
    //   [Buffer.from("global")],
    //   program.programId
    // );
    // const connection = new anchor.web3.Connection(
    //   `https://solana-devnet.g.alchemy.com/v2/xOw_vw5ZnqjQsPr9ILoZcA98LV5rrGVJ`
    // );
    // const { blockhash } = await connection.getLatestBlockhash();
    // console.log(blockhash);

    // let tx = await program.methods.initialize(bump)
    //   .accounts({
    //     bridgeConfig: bridgeConfig,
    //     authority: provider.wallet.publicKey,
    //     systemProgram: anchor.web3.SystemProgram.programId,
    //   })
    //   .rpc();
    // console.log("tx=>", tx);

    // const multisigInfo = await program.account.configInfo.fetch(bridgeConfig);
    // console.log("Multisig info initialized:", multisigInfo);
  });

  it("Change admin", async () => {
    return;
    // [bridgeConfig, bump] = await anchor.web3.PublicKey.findProgramAddressSync(
    //   [Buffer.from("global")],
    //   program.programId
    // );

    // let instruction = await program.methods.changeAdmin(new anchor.web3.PublicKey(""))
    //   .accounts({
    //     admin: provider.wallet.publicKey,
    //     bridgeConfig: bridgeConfig,
    //   })
    //   .instruction();

    // const transaction = new anchor.web3.Transaction().add(instruction);
    // transaction.feePayer = provider.wallet.publicKey;
    // // 获取最近区块哈希
    // const { blockhash } = await provider.connection.getLatestBlockhash();
    // transaction.recentBlockhash = blockhash;
    // transaction.sign(feePayer, programKeypair);

    // 发送交易
    // const signature = await provider.sendAndConfirm(transaction);
    // console.log('Transaction signature:', signature);


    const multisigInfo = await program.account.configInfo.fetch(bridgeConfig);
    console.log("Multisig info initialized:", multisigInfo);
  });

  it("withdrow fee", async () => {
    let [messageFee, bump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultFee")],
      program.programId
    );
    console.log("messageFee", messageFee.toBase58())

    let signature = await program.methods.withdrawFee(new anchor.BN(100000))
      .accounts({
        superAdmin: provider.wallet.publicKey,
      })
      .rpc();
    console.log(signature)
  });
});
