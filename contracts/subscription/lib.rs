//! Subscription is a simple smart contract which allows for periodical subscriptions to a resource.
//!
//! # Overview
//!
//! Callers may pay `SUBSCRIPTION_COST` tokens to obtain `SUBSCRIPTION_TIME` hours of media time for
//! the current month. Subsequent payments will not increase the allocated subscription time for the
//! current month, but instead ensure that they have subscriptions time allocated for the next month.
//!
//! The contract owner is able to consume a user's allocated subscription time.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod subscription {

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{collections::HashMap as StorageHashMap, lazy::Lazy};

    use ink_env::call::FromAccountId;
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use privi::Privi;

    /// Number of tokens required to purchase a subscription.
    pub const SUBSCRIPTION_COST: u128 = 100; // TODO the spec does not define this to be configurable, but probably should.

    /// Number of hours allocated to user after purchasing a subscription.
    pub const SUBSCRIPTION_TIME: u128 = 50; // TODO the spec does not define this to be configurable, but probably should.

    /// The subscription information of a user.
    #[derive(
        Clone,
        Debug,
        Default,
        PartialEq,
        Eq,
        SpreadLayout,
        PackedLayout,
        scale::Encode,
        scale::Decode,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Subscription {
        // Hours of free media consumption left.
        hours: u128,

        // The number of months the user has payed for.
        months_payed: u128,

        // The Timestamp of the payment
        timestamp: Timestamp,
    }

    /// Subscription smart contract storage.
    #[ink(storage)]
    pub struct Subscriber {
        owner: Lazy<AccountId>,
        token: Lazy<Privi>,
        subscriptions: StorageHashMap<AccountId, Subscription>,
        subscription_max_age: Lazy<Timestamp>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum SubscribeError {
        Erc20(privi::Error),
        InsufficientFunds,
    }

    impl From<privi::Error> for SubscribeError {
        fn from(err: privi::Error) -> Self {
            Self::Erc20(err)
        }
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AuthError {
        IsNotOwner,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ConsumeHoursError {
        AuthError(AuthError),
        InsufficientHours,
    }

    impl From<AuthError> for ConsumeHoursError {
        fn from(err: AuthError) -> Self {
            Self::AuthError(err)
        }
    }

    /// Subscription events are emitted after each purchased subscription, containing the new
    /// state.
    #[ink(event)]
    pub struct SubscriptionEvent {
        #[ink(topic)]
        account: AccountId,
        subscription: Subscription,
    }

    /// Consumption events are emitted after each media consumption, containing the remaining and
    /// consumed hours.
    #[ink(event)]
    pub struct ConsumptionEvent {
        #[ink(topic)]
        account: AccountId,
        remaining: u128,
        consumed: u128,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum TransferError {
        Erc20(privi::Error),
        AuthError(AuthError),
    }

    impl From<privi::Error> for TransferError {
        fn from(err: privi::Error) -> Self {
            Self::Erc20(err)
        }
    }

    impl From<AuthError> for TransferError {
        fn from(err: AuthError) -> Self {
            Self::AuthError(err)
        }
    }

    impl Subscriber {
        #[ink(constructor)]
        pub fn new(erc20_address: AccountId, subscription_max_age: Timestamp) -> Self {
            let erc20 = Privi::from_account_id(erc20_address);

            let owner = Self::env().caller();

            Self {
                subscriptions: StorageHashMap::new(),
                token: Lazy::new(erc20),
                owner: Lazy::new(owner),
                subscription_max_age: Lazy::new(subscription_max_age),
            }
        }

        /// Purchases `num_months` of subscription for `account`. If the account already has a
        /// subscription, no extra hours are purchased, but instead more months are added. Note
        /// that hours do not roll over, but are reset after each month.
        #[ink(message)]
        pub fn subscribe_n(
            &mut self,
            account: AccountId,
            num_months: u128,
        ) -> Result<u128, SubscribeError> {
            let caller = self.env().caller();
            let account_id = self.env().account_id();

            let cost = num_months.saturating_mul(SUBSCRIPTION_COST);

            if self.token.balance_of(caller) < cost
                || self.token.allowance(caller, account_id) < cost
            {
                return Err(SubscribeError::InsufficientFunds);
            }

            self.token.transfer_from(caller, account_id, cost)?;

            let now = self.env().block_timestamp();

            let mut subscription = self.subscriptions.entry(account).or_insert(Subscription {
                hours: SUBSCRIPTION_TIME,
                months_payed: 0,
                timestamp: now,
            });

            subscription.months_payed += 1;

            let subscription = subscription.clone();

            self.env().emit_event(SubscriptionEvent {
                account,
                subscription,
            });

            Ok(50)
        }

        /// Obtains the subscription info for the provided account.
        #[ink(message)]
        #[must_use]
        pub fn subscription_info(&self, account: AccountId) -> Subscription {
            self.subscriptions
                .get(&account)
                .map(Clone::clone)
                .unwrap_or_default()
        }

        /// Reduce the number of media hours left for account. If a month has passed since last
        /// purchase, hours are reset, or newly allocated if the user has paid for multiple months.
        ///
        /// # Restrictions
        ///
        /// May only be called by the contract owner.
        #[ink(message)]
        pub fn consume(
            &mut self,
            account: AccountId,
            hours: u128,
        ) -> Result<Subscription, ConsumeHoursError> {
            self.assert_is_owner()?;

            let now = self.env().block_timestamp();

            let subscription = self
                .subscriptions
                .get_mut(&account)
                .ok_or(ConsumeHoursError::InsufficientHours)?;

            if subscription.timestamp + *self.subscription_max_age <= now {
                let months_passed =
                    now.saturating_sub(subscription.timestamp) / *self.subscription_max_age;
                subscription.months_payed = subscription
                    .months_payed
                    .saturating_sub(months_passed.into());
                if subscription.months_payed > 0 {
                    subscription.hours = 50;

                    // This may actually leads to a timestamp drift, if consume is not called every day.
                    // Alternatively, another method call could be implemented which updates all subscription
                    // timestamps, so that the contract owner may mass update all subscriptions.
                    subscription.timestamp = now;
                }
            }

            if subscription.hours < hours {
                return Err(ConsumeHoursError::InsufficientHours);
            }

            subscription.hours = subscription.hours.saturating_sub(hours);
            let subscription = subscription.clone();

            self.env().emit_event(ConsumptionEvent {
                account,
                remaining: subscription.hours,
                consumed: hours,
            });

            Ok(subscription)
        }

        /// Transfers funds from the token wallet to the provided account
        ///
        /// # Restrictions
        ///
        /// May only be called by the contract owner.
        #[ink(message)]
        pub fn transfer_out(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), TransferError> {
            self.assert_is_owner()?;
            self.token.transfer(account, amount)?;
            Ok(())
        }

        fn assert_is_owner(&self) -> Result<(), AuthError> {
            let caller = self.env().caller();
            if *self.owner != caller {
                return Err(AuthError::IsNotOwner);
            }
            Ok(())
        }
    }
}
