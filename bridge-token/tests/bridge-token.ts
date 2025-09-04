import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BridgeToken } from "../target/types/bridge_token";
import { base64, bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccountIdempotentInstruction, createCloseAccountInstruction, createSyncNativeInstruction, getAssociatedTokenAddress, NATIVE_MINT, TOKEN_PROGRAM_ID, } from "@solana/spl-token"
import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

describe("bridge-token", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BridgeToken as Program<BridgeToken>;
  const bridge_program_id = new PublicKey("DwZPNgo4Qtxw2PcbSVgs16kVzynhBLHMF1iz9Js1efY5");
  const msg_program_id = new PublicKey("5Dwsb6syuTtb6LEBw8CdbBdZS4Pfd8jWjiS3nmYRY6KP");

  it("Is bridge!", async () => {
    // let relaction = await anchor.web3.PublicKey.findProgramAddressSync([new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0, 221]), new Uint8Array([198, 151, 18, 5, 126, 99, 75, 235, 201, 171, 2, 116, 93, 45, 105, 238, 115, 142, 62, 180, 245, 211, 1, 137, 169, 172, 191, 142, 8, 251, 130, 62])], new anchor.web3.PublicKey("DwZPNgo4Qtxw2PcbSVgs16kVzynhBLHMF1iz9Js1efY5"))
    // console.log(relaction[0].toBase58())

    // 下面参数是EDS链的
    const to_chain = {
      chainType: 0,
      chainId: new anchor.BN(221),
    };
    const serializedChain = combain_chain(to_chain);
    console.log(serializedChain);
    // let to_token = new anchor.web3.PublicKey("SoLnoppLP91g3NWeASXYKamdCiB9xqj8i6k6ys9jnpd");//sol
    let to_token = new anchor.web3.PublicKey("ENDLESSsssssssssssssssssssssssssssssssssssss");//eds
    let to_who = new anchor.web3.PublicKey("ECi3osWUM85BpnMsLAZYGevE3fJ73vCKvmmW9EA5GRT8");
    let all_amount = new anchor.BN(5000000000)//5sol
    let upload_gas_fee = new anchor.BN(1000000000)//5sol

    // SOL链参数
    // let token_mint = NATIVE_MINT;//wsol
    let token_mint = new PublicKey("EDS1VYanZME3G8F7R9MC6S7CUZwKQwn6U1BfDMfaxVx7");//eds
    let sender = provider.wallet.publicKey;
    console.log("sender ", sender.toBase58());
    let sender_token = await getAssociatedTokenAddress(token_mint, sender, true);
    console.log("sender_token ", sender_token.toBase58());

    let transactions = new Transaction();
    let close_ata = false;
    // 原生币需要转换
    if (token_mint == NATIVE_MINT) {
      try {
        let account = await provider.connection.getAccountInfo(sender_token);
        console.log("account", account);
        transactions.add(SystemProgram.transfer({
          fromPubkey: sender,
          toPubkey: sender_token,
          lamports: Number(all_amount),
        }));
        transactions.add(createSyncNativeInstruction(sender_token));
      } catch (error) {
        transactions.add(createAssociatedTokenAccountIdempotentInstruction(
          sender,
          sender_token,
          sender,
          NATIVE_MINT
        ));
        transactions.add(SystemProgram.transfer({
          fromPubkey: sender,
          toPubkey: sender_token,
          lamports: Number(all_amount),
        }));
        transactions.add(createSyncNativeInstruction(sender_token));
        close_ata = true;
      }
    }

    // =========合约关联账户=========
    // bridge 合约
    let [bridge_authority] = PublicKey.findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode("bridge"))],
      bridge_program_id
    );
    console.log("bridge_authority ", bridge_authority.toBase58());
    let fund_pool = await getAssociatedTokenAddress(token_mint, sender, true);
    console.log("fund_pool ", fund_pool.toBase58());
    let [pool_account] = PublicKey.findProgramAddressSync(
      [token_mint.toBuffer(), Buffer.from(anchor.utils.bytes.utf8.encode("pool"))],
      bridge_program_id
    );
    console.log("pool_account ", pool_account.toBase58());
    let [token_relation] = PublicKey.findProgramAddressSync(
      [combain_chain(to_chain), to_token.toBuffer()],
      bridge_program_id
    );
    console.log("token_relation ", token_relation.toBase58());
    let [chain_relation] = PublicKey.findProgramAddressSync(
      [combain_chain(to_chain), Buffer.from(anchor.utils.bytes.utf8.encode("chain_relation"))],
      bridge_program_id
    );
    console.log("chain_relation ", chain_relation.toBase58());
    //msg 合约
    let [to_chain_nonce_account] = PublicKey.findProgramAddressSync(
      [combain_chain(to_chain), Buffer.from(anchor.utils.bytes.utf8.encode("toNonce"))],
      msg_program_id
    );
    console.log("to_chain_nonce_account ", to_chain_nonce_account.toBase58());
    let [message_fee] = PublicKey.findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode("vaultFee"))],
      msg_program_id
    );
    console.log("message_fee ", message_fee.toBase58());
    let [bridge_config] = PublicKey.findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode("global"))],
      msg_program_id
    );
    console.log("bridge_config ", bridge_config.toBase58());

    let toToken: number[] = new Array(32).fill(0);
    toToken = to_token.toBytes() as any;
    let toWho: number[] = new Array(32).fill(0);
    toWho = to_who.toBytes() as any;
    // 跨链指令
    transactions.add(await program.methods.bridgeProposal({
      chainType: to_chain.chainType,
      chainId: to_chain.chainId
    }, toToken, toWho, all_amount, upload_gas_fee)
      .accounts({
        sender,
        tokenMint: token_mint,
        senderToken: sender_token,
        fundPool: fund_pool,
        poolAccount: pool_account,
        bridgeAuthority: bridge_authority,
        tokenRelation: token_relation,
        chainRelation: chain_relation,
        toChainNonceAccount: to_chain_nonce_account,
        messageFee: message_fee,
        bridgeConfig: bridge_config,
        bridgeCoreProgram: msg_program_id,
        programId: bridge_program_id,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID
      }).instruction());

    let a = await program.methods.bridgeProposal({
      chainType: to_chain.chainType,
      chainId: to_chain.chainId
    }, toToken, toWho, all_amount, upload_gas_fee)
      .accounts({
        sender,
        tokenMint: token_mint,
        senderToken: sender_token,
        fundPool: fund_pool,
        poolAccount: pool_account,
        bridgeAuthority: bridge_authority,
        tokenRelation: token_relation,
        chainRelation: chain_relation,
        toChainNonceAccount: to_chain_nonce_account,
        messageFee: message_fee,
        bridgeConfig: bridge_config,
        bridgeCoreProgram: msg_program_id,
        programId: bridge_program_id,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID
      }).instruction();
    console.log("instruction_data==>", base64.encode(a.data));

    if (close_ata) {
      transactions.add(createCloseAccountInstruction(sender_token, sender, sender))
    }

    let latestBlockhash = await provider.connection.getLatestBlockhash();
    transactions.recentBlockhash = latestBlockhash.blockhash;
    transactions.feePayer = sender;
    let result = await provider.connection.simulateTransaction(transactions);
    console.log(result);
  });
});

// 将结构体转换为 Uint8Array
function combain_chain(chain: { chainType: number; chainId: anchor.BN }): Uint8Array {
  const chainBuffer = new Uint8Array(9);
  chainBuffer[0] = chain.chainType;
  chainBuffer.set(chain.chainId.toArray('be', 8), 1); // 将 BN 转换为大端格式的 u64, 并从索引1开始填充
  return chainBuffer;
}