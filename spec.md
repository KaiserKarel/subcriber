Hi Karel,

Nice to chat with you this morning, after some consideration I'd like to
develop a subscription based ink! Smart Contract, the logic is as follows:

1. You first write and deploy a ERC20 token contract, we can call it PRIV;

2. Users will use the above PRIV token as a payment token, he send in
   his PRIV token to a Service contract;

3. The Service Contract will take, let's say 100 PRIV payment, and
   return the payer a u128 number representing the hours of free media
   consumption;

4. The payment is a recurring payment, e.g. the current payment will
   expire in a month, after that the u128 number will be auto reset to 0;
   if the user pays the PRIVI before expiry, new number will be set to the
   next month, and the current month's number will remain until expiry;

I'd like you to develop such "monthly subscription" contracts in
Substrate ink!. Some references:

ink! contract examples

https://github.com/paritytech/ink

ink! contract development tool - RedSpot

Redspot is useful when you do cross contract calling

https://github.com/patractlabs/redspot

For any questions, feel free to DM me on Slack or email.

Thanks,

Mike
