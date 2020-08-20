use std::collections::HashMap;
use std::time::SystemTime;

use blake2::{Blake2b, Digest};

/// Blockchain container
#[derive(Debug, Clone)]
pub struct Blockchain {
    /// Store for all the blocks which are accepted
    pub blocks: Vec<Block>,

    /// Store for hashmap with AccountId and Account associated with it.
    pub accounts: HashMap<String, Account>,

    /// Store for transactions whick are pending in the moment.
    pending_transactions: Vec<Transaction>,
}

/// Blockchain methods
impl Blockchain {
    /// Create new chain
    pub fn new() -> Self {
        Blockchain {
            blocks: Vec::new(),
            accounts: HashMap::new(),
            pending_transactions: Vec::new(),
        }
    }

    /// Get hash of the last block
    pub fn get_last_block_hash(&self) -> Option<String> {
        if self.blocks.is_empty() {
            return None;
        }

        self.blocks[self.blocks.len() - 1].hash.clone()
    }
    /// Append block
    pub fn append_block(&mut self, block: Block) -> Result<(), String> {
        // Check if block is first in the chain(genesis)
        let is_genesis = self.blocks.is_empty();

        if block.prev_hash != self.get_last_block_hash() {
            return Err("The new block has to point to the previous block".into());
        }

        if block.get_transaction_count() == 0 {
            return Err("There has to be at least one transactio inside the block!".into());
        }

        // TODO: refactor to something more resource friendly
        let old_state = self.accounts.clone();

        // Execute each transaction and rollback if something went wrong
        for (i, transaction) in block.transactions.iter().enumerate() {
            // Execute the transaction
            if let Err(err) = transaction.execute(self, &is_genesis) {
                // Recover state in case of fail
                self.accounts = old_state;

                // Reject current block
                return Err(format!(
                    "Could not execute transaction {} because of '{}. Rolling back",
                    i + 1,
                    err
                ));
            }
        }

        self.blocks.push(block);

        Ok(())
    }
    pub fn check_validity(&self) -> Result<(), String> {
        for (block_num, block) in self.blocks.iter().enumerate() {
            // Check if block saved hash matches to calculated hash
            if !block.verify_own_hash() {
                return Err(format!(
                    "Stored hash for Block #{} \
                    does not match calculated hash",
                    block_num + 1
                ));
            }

            // Check previous black hash points to actual previous block
            if block_num == 0 {
                // Genesis block should point to nowhere
                if block.prev_hash.is_some() {
                    return Err("The genesis block has a previous hash set which \
                     it shouldn't Code :394823098"
                        .into());
                }
            } else {
                // Non genesis blocks should point to previous blocks hash (which is validated before)
                if block.prev_hash.is_none() {
                    return Err(format!("Block #{} has no previous hash set", block_num + 1));
                }

                // Store the values locally to use them within the error message on failure
                let prev_hash_proposed = block.prev_hash.as_ref().unwrap();
                let prev_hash_actual = self.blocks[block_num - 1].hash.as_ref().unwrap();

                if block.prev_hash != self.blocks[block_num - 1].hash {
                    return Err(format!(
                        "Block #{} is not connected to previous block (Hashes do \
                    not match. Should be `{}` but is `{}`)",
                        block_num, prev_hash_proposed, prev_hash_actual
                    ));
                }
            }

            // Check if transactions are signed correctly
            for (transaction_num, transaction) in block.transactions.iter().enumerate() {
                if transaction.is_signed() && !transaction.check_signature() {
                    return Err(format!(
                        "Transaction #{} for Block #{} has an invalid signature",
                        transaction_num + 1,
                        block_num + 1
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Block
#[derive(Debug, Clone)]
pub struct Block {
    pub hash: Option<String>,
    pub prev_hash: Option<String>,
    pub unqnum: i128,
    pub(crate) transactions: Vec<Transaction>,
}

/// Block methods
impl Block {
    pub fn new(prev_hash: Option<String>) -> Self {
        Block {
            unqnum: 0,
            hash: None,
            prev_hash,
            transactions: Vec::new(),
        }
    }

    /// Changes the unqnum number and updates the hash
    pub fn set_unqnum(&mut self, unqnum: i128) {
        self.unqnum = unqnum;
        self.update_hash();
    }

    /// Will calculate the hash of the whole block including transactions Blake2 hasher
    pub fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Blake2b::new();

        for transaction in self.transactions.iter() {
            hasher.update(transaction.calculate_hash())
        }

        let block_as_string = format!("{:?}", (&self.prev_hash, &self.unqnum));
        hasher.update(&block_as_string);

        Vec::from(hasher.finalize().as_ref())
    }

    /// Appends a transaction to the queue
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
        self.update_hash();
    }

    /// Will return the amount of transactions
    pub fn get_transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Will update the hash field by including all transactions currently inside
    /// the public modifier is only for the demonstration of attacks
    pub(crate) fn update_hash(&mut self) {
        self.hash = Some(byte_vector_to_string(&self.calculate_hash()));
    }

    /// Checks if the hash is set and matches the blocks interna
    pub fn verify_own_hash(&self) -> bool {
        if self.hash.is_some() && // Hash set
            self.hash.as_ref().unwrap().eq(
                &byte_vector_to_string(
                    &self.calculate_hash()))
        {
            // Hash equals calculated hash

            return true;
        }
        false
    }
}
/// Transaction
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Unique number
    unqnum: u128,

    /// Accound ID
    from: String,

    /// Time the transaction was created
    created_at: SystemTime,

    /// Transaction type and it's information
    pub(crate) record: TransactionData,

    /// Signature of the message (basic auth)
    signature: Option<String>,
}

impl Transaction {
    pub fn new(from: String, transaction_data: TransactionData, unqnum: u128) -> Self {
        Transaction {
            from,
            unqnum,
            record: transaction_data,
            created_at: SystemTime::now(),
            signature: None,
        }
    }

    /// Will change the world state according to the transactions commands
    pub fn execute<T: WorldState>(
        &self,
        world_state: &mut T,
        is_initial: &bool,
    ) -> Result<(), &'static str> {
        // Check if sending user does exist (no one not on the chain can execute transactions)
        if let Some(_account) = world_state.get_account_by_id(&self.from) {
            // Do some more checkups later on...
        } else if !is_initial {
            return Err("Account does not exist");
        }

        // match is like a switch (pattern matching) in C++ or Java
        // We will check for the type of transaction here and execute its logic
        match &self.record {
            TransactionData::CreateUserAccount(account) => {
                world_state.create_account(account.into(), AccountType::User)
            }

            TransactionData::CreateTokens { receiver, amount } => {
                if !is_initial {
                    return Err("Token creation is only available on initial creation");
                }
                // Get the receiving user (must exist)
                if let Some(account) = world_state.get_account_by_id_mut(receiver) {
                    account.tokens += *amount;
                    Ok(())
                } else {
                    Err("Receiver Account does not exist")
                }
            }

            TransactionData::TransferTokens { to, amount } => {
                let recv_tokens: u128;
                let sender_tokens: u128;

                if let Some(recv) = world_state.get_account_by_id_mut(to) {
                    // Be extra careful here, even in the genesis block the sender account has to exist
                    recv_tokens = recv.tokens;
                } else {
                    return Err("Receiver Account does not exist!");
                }

                if let Some(sender) = world_state.get_account_by_id_mut(&self.from) {
                    sender_tokens = sender.tokens;
                } else {
                    return Err("That account does not exist!");
                }

                let balance_recv_new = recv_tokens.checked_add(*amount);
                let balance_sender_new = sender_tokens.checked_sub(*amount);

                if balance_recv_new.is_some() && balance_sender_new.is_some() {
                    world_state
                        .get_account_by_id_mut(&self.from)
                        .unwrap()
                        .tokens = balance_sender_new.unwrap();
                    world_state.get_account_by_id_mut(to).unwrap().tokens =
                        balance_recv_new.unwrap();
                    Ok(())
                } else {
                    Err("Overspent or Arithmetic error")
                }
            }

            _ => {
                // Not implemented transaction type
                Err("Unknown Transaction type (not implemented)")
            }
        }
    }

    /// Will calculate the hash using Blake2 hasher
    pub fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Blake2b::new();
        let transaction_as_string = format!(
            "{:?}",
            (&self.created_at, &self.record, &self.from, &self.unqnum)
        );

        hasher.update(&transaction_as_string);
        Vec::from(hasher.finalize().as_ref())
    }

    /// Will hash the transaction and check if the signature is valid
    /// (i.e., it is created by the owners private key)
    /// if the message is not signed it will always return false
    pub fn check_signature(&self) -> bool {
        if !(self.is_signed()) {
            return false;
        }

        //@TODO check signature
        false
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}
/// TransactionData
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionData {
    /// Store for new user account
    CreateUserAccount(String),

    /// Method for changing or creating value into an account
    ChangeStoreValue { key: String, value: String },

    /// Method for moving tokens from one owner to another
    TransferTokens { to: String, amount: u128 },

    /// Method for creating tokens
    CreateTokens { receiver: String, amount: u128 },
}

/// Account
#[derive(Debug, Clone)]
pub struct Account {
    /// Store for user's data
    store: HashMap<String, String>,

    /// Account type
    account_type: AccountType,

    /// Amount of tokens
    tokens: u128,
}

/// Account methods
impl Account {
    /// Constructor
    pub fn new(account_type: AccountType) -> Self {
        Self {
            tokens: 0,
            account_type,
            store: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
// TODO: implement more types such as Validator(to check validation of blocks in the chain)
/// Account type
pub enum AccountType {
    /// A common user account
    User,
}

/// World State
pub trait WorldState {
    /// Method for getting user ids
    fn get_user_ids(&self) -> Vec<String>;

    /// Method for returning an account given it's id if is available
    fn get_account_by_id_mut(&mut self, id: &str) -> Option<&mut Account>;

    fn get_account_by_id(&self, id: &str) -> Option<&Account>;

    /// Method for adding a new account
    fn create_account(&mut self, id: String, account_type: AccountType)
        -> Result<(), &'static str>;
}

impl WorldState for Blockchain {
    fn get_account_by_id_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.get_mut(id)
    }

    fn get_account_by_id(&self, id: &str) -> Option<&Account> {
        self.accounts.get(id)
    }
    fn get_user_ids(&self) -> Vec<String> {
        self.accounts.keys().cloned().collect()
    }

    fn create_account(
        &mut self,
        id: String,
        account_type: AccountType,
    ) -> Result<(), &'static str> {
        if !self.get_user_ids().contains(&id) {
            let acc = Account::new(account_type);
            self.accounts.insert(id, acc);
            Ok(())
        } else {
            Err("User already exists!")
        }
    }
}

fn byte_vector_to_string(arr: &[u8]) -> String {
    arr.iter().map(|&c| c as char).collect()
}
