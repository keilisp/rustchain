use std::borrow::BorrowMut;

mod blockchain;
use blockchain::{Block, Blockchain, Transaction, TransactionData};

fn main() {
    println!("Playin' with rustchain");

    // Creating new chain
    let mut chain = Blockchain::new();
    // Setting genesis block
    let mut genesis = Block::new(None);

    let initial_users = vec!["John", "Mereep"];

    for user in initial_users {
        let uniq_num = Transaction::generate_unqnum();

        println!("UNIQ NUMBER: {}", uniq_num);

        let create_transaction = Transaction::new(
            user.into(),
            TransactionData::CreateUserAccount(user.into()),
            uniq_num, // 0,
        );

        let token_action = Transaction::new(
            user.into(),
            TransactionData::CreateTokens {
                receiver: user.into(),
                amount: 10_000,
            },
            uniq_num, // 0,
        );

        genesis.add_transaction(create_transaction);
        genesis.add_transaction(token_action);
    }

    let mut res = chain.append_block(genesis);
    println!("Genesis block was added: {:?}", res);
    println!("Full blockchain: ");
    println!("{:#?}", chain);

    // Transfer 100 tokens from John to Mereep
    let mut block2 = Block::new(chain.get_last_block_hash());
    block2.add_transaction(Transaction::new(
        "John".into(),
        TransactionData::TransferTokens {
            to: "Mereep".into(),
            amount: 100,
        },
        0,
    ));
    // FIXME: do something about dummy 0 in Block::unqnum
    block2.set_unqnum(324);

    res = chain.append_block(block2);
    println!("Block2 added: {:?}", res);

    let mut block3 = Block::new(chain.get_last_block_hash());
    block3.add_transaction(Transaction::new(
        "Mereep".into(),
        TransactionData::TransferTokens {
            to: "John".into(),
            amount: 1000,
        },
        0,
    ));

    block3.set_unqnum(95);

    res = chain.append_block(block3);
    println!("Block3 added: {:?}", res);

    println!("Full blockchain printout");
    println!("{:#?}", chain);
    // println!("Blockchain valid: {:?}", chain.check_validity());

    let mut chain_attack = chain.clone();

    let transaction_data = chain_attack.blocks[1].transactions[0].borrow_mut();

    // Change the amount value of the transaction inside the chain
    if let TransactionData::TransferTokens {
        to: _,
        ref mut amount,
    } = transaction_data.record.borrow_mut()
    {
        *amount = 1000; // Changing the value in place
    }

    println!(
        "Is the Blockchain still valid? {:#?}",
        chain_attack.check_validity()
    );
}
