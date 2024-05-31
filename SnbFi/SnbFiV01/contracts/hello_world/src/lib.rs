#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol, Address};
use soroban_sdk::Map;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Frequency {
    DAY, 
    WEEK , 
    MONTH
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Subscriber {
    // winner of a specific iteration, if not it will be 0
    pub winner_at_iter: u32,
    //prize money won by the subscriber
    pub prize_money: u32,
    // previous due amount of the subscriber, not including the current due amount
    pub prev_due_amount: u32
}

#[derive(Clone,Debug, Eq, PartialEq)]
#[contracttype]
pub struct PoolParams {
    // number of subscribers in the pool
    pub no_of_subs: u32,
    // frequency of the pool
    pub frequency: Frequency,
    // subscription amount
    pub sub_amount : u32,
    // pool was initiated by the owner
    pub pool_owner : Address
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PoolIterationParams {
    // current iteration of the pool, it will 0 before it starts, once it starts it will be incremented by 1
    pub current_iteration: u32,
    // amount collected so far in the current iteration
    pub amount_collected: u32,
    // winner of the current iteration
    pub winner: Address,
    //prize money to winner
    pub prize_money: u32,
    // dividend amount in the pool
    pub dividend_amount: u32
}


pub trait SnbPoolTrait {

    fn get_state(env: Env) -> State;

    //Initialise pool 
    fn initialize(e: Env,  user: Address, no_of_subs: u32, amount: u32, frequency: Frequency);

    //join the pool
    fn join(e: Env);

    //Set pool winner
    fn set_pool_winner(e: Env, iteration: u32, prize_amount: u32, subscriber: Address);

    //Get pool winner of a specific iteration
    fn get_pool_winner(e: Env, iteration: u32) -> Address;

    //Get subscriber details
    fn get_subscriber_details(e: Env, subscriber_address: Address) -> Subscriber;

    //start a new iteration
    fn start_new_iteration(e: Env, iteration: u32 );

}

 

pub trait Reputation {
    fn addReputation(e: Env,  subscriber: Address, reputation: u32);
    fn getReputation(e: Env,  subscriber: Address) -> u32;
}

const STATE: Symbol = symbol_short!("STATE");
const INTIALIZED: Symbol = symbol_short!("INITD");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
      pub pool_params: PoolParams,
      pub current_iteration: u32,
      // A map data structure from subscriber address to subscriber
      pub subcriber_map: Map<Address, Subscriber>,
  
      // A map data structure from iteration to PoolIterationParams
      pub pool_iteration_map: Map<u32, PoolIterationParams>  
}


#[contract]
pub struct HelloContract;

#[contractimpl]
impl SnbPoolTrait for HelloContract {

    fn get_state(env: Env) -> State {
        //let mut sampleAddress = String::from("GCCVPYFOHY7ZB7557JKENAX62LUAPLMGIWNZJAFV2MITK6T32V37KEJU");
        env.storage().instance().get(&STATE).unwrap(
            panic!("State not found")
        )
    }

    fn initialize(e: Env, user: Address, no_of_subs: u32, amount: u32, frequency: Frequency) {
        //Check if the pool is already initialized
        let mut initialized = e.storage().instance().get(&INTIALIZED);
        if initialized {
            panic!("Pool is already initialized");
        }
        let mut state = Self::get_state(e.clone());
        user.require_auth();
         //Create a PoolParams instance
         let pool_params = PoolParams {
            no_of_subs: no_of_subs,
            frequency: frequency,
            sub_amount: amount,
            pool_owner: user
        };
        state.pool_params = pool_params;

        //Save the pool_params in the storage
        //e.storage().instance().set(&Self::pool_params{ no_of_subs: no_of_subs, frequency: frequency, sub_amount: amount, pool_owner: e.caller() });
        

        //Add owner to the subscriber list
        let subscriber = Subscriber {
            winner_at_iter: 0,
            prev_due_amount: 0,
            prize_money: 0
        };
        state.subscriber_map.insert(e.caller(), subscriber);
        state.initialized = true;
        
        //save the state in the storage
        e.storage().instance().set(&STATE, state);
    }

    fn join(e: Env) {
        // check if the subscriber is already in the pool. Only one address allowed in apool
        if &Self::subscriber_map.contains_key(&e.caller()) {
            panic!("Subscriber is already in the pool");
        }

        // Create Subscriber instance with default values
        let subscriber = Subscriber {
            winner_at_iter: 0,
            prev_due_amount: 0
        };
        //Save the subscriber in the subscriber_map
        &Self::subscriber_map.insert(e.caller(), subscriber);
    }

    fn set_pool_winner(e: Env, iteration: u32, prize_amount: u32, subscriber: Address) {
        // get the subscriber details for given address
        let subr:Subscriber = &Self::subscriber_map.get(&subscriber).unwrap();
        // check if the subscriber is already marked as a winner
        if subr.winner_at_iter != 0 {
            panic!("Subscriber is already a winner");
        }
        //set iterarion for this subscriber
        subr.winner_at_iter = iteration;
        //set prize money for this subscriber
        subr.prize_money = prize_amount;
        //save the subscriber in the subscriber_map
        &Self::subscriber_map.insert(&subscriber, subr);

        // get the pool iteration details
        let pool_iteration:PoolIterationParams = &Self::pool_iteration_map.get(&iteration);
        // set the winner for the given iteration
        pool_iteration.winner = subscriber;
        // set the prize money for the given iteration
        pool_iteration.prize_money = prize_amount;
        //set iteration to the pool iteration map
        pool_iteration.dividend = pool_iteration.amount_collected - prize_amount;
        &Self::pool_iteration_map.insert(&iteration, pool_iteration);

        //save subscriber map
        //e.storage().instance().set(&SubscriberMap, val::subscriber_map);

        //save the pool iteration in the storage
        //e.storage().instance().set(&PoolIterationMap, val::pool_iteration_map);
    }

    fn get_pool_winner(e: Env, iteration: u32) -> Address {
        //return iteration winner
        return  &Self::pool_iteration_map.get(&iteration).unwrap().winner
    }

    fn get_subscriber_details(e: Env, subscriber_address: Address) -> Subscriber {
        //get subscriber details by address
        return  &Self::subscriber_map.get(&subscriber_address).unwrap();
    }

    //start new iteration
    fn start_new_iteration(e: Env, iteration: u32) {
        //create a new instance of PoolIterationParams
        let pool_iteration = PoolIterationParams {
            current_iteration: iteration,
            amount_collected: 0,
            winner: Address::default(),
            prize_money: 0,
            dividend_amount: 0
        };
        //save the pool iteration in the pool_iteration_map
        &Self::pool_iteration_map.insert(iteration, pool_iteration);
        //save the pool iteration map in the storage
        //e.storage().instance().set(&PoolIterationMap, val::pool_iteration_map);
    }
}


mod test;
