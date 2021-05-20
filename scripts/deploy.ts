import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { getSigners, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
  await api.isReady;

  const signers = await getSigners()
  const alice = signers[0]

  let contractFactory = await getContractFactory('privi', alice);
  const balance = await api.query.system.account(alice.address);
  console.log('Balance: ', balance.toHuman());

  let privi = await contractFactory.deployed('new', '1000000', {
    gasLimit: '200000000000',
    value: '100000000000'
  });

  console.log('');
  console.log(
    'Privi Deploy successfully. The contract address: ',
      privi.address.toString()
  );

  contractFactory = await getContractFactory('subscription', alice);

  let contract = await contractFactory.deployed('new', "0xfbba0d643995681566a0d7d18084434bfea06ed952aa5eb51a8a743d161daff7", 0, {
    gasLimit: '200000000000',
    value: '100000000000'
  });

  console.log('');
  console.log(
      'Subscriber Deploy successfully. The contract address: ',
      contract.address.toString()
  );

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
