import BN from 'bn.js';
import { expect } from 'chai';
import { patract, network, artifacts } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;

const { api, getSigners } = network;

interface Cfg {
    subscriptionMaxAge: number
}

describe('Subscription', () => {
    after(() => {
        return api.disconnect();
    });

    async function setup(cfg?: Cfg) {
        let subscriptionMaxAge = (cfg?.subscriptionMaxAge === undefined) ? 100000 : cfg?.subscriptionMaxAge;


        const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
        const signers = await getSigners();
        const Alice = signers[0];
        const sender = await getRandomSigner(Alice, one.muln(1000000));
        const Bob = signers[1];

        const priviContractFactory = await getContractFactory('privi', sender);
        const privi = await priviContractFactory.deploy('new', '1000000000');

        const contractFactory = await getContractFactory('subscription', sender);
        const receiver = await getRandomSigner();
        const contract = await contractFactory.deployed('new', privi.address, subscriptionMaxAge, {
            gasLimit: '200000000000',
            value: '100000000000'
        });

        const abi = artifacts.readArtifact('subscription');
        return { sender, contractFactory, contract, privi, abi, receiver, Alice, one, Bob };
    }

    it('Correctly sets the initial subscription', async () => {
        const { contract, privi } = await setup();
        await privi.approve(contract.address, 10000);
        await contract.tx.subscribeN(contract.address, 1);
        let result = await contract.query.subscriptionInfo(contract.address);
        expect(result.output.hours).to.equal(50);
        expect(result.output.months_payed).to.equal(1);
    });

    it('Consumes hours correctly', async () => {
        const { contract, privi } = await setup();
        await privi.approve(contract.address, 10000);
        await contract.tx.subscribeN(contract.address, 1);
        await contract.tx.consume(contract.address, 50);
        let result = await contract.query.subscriptionInfo(contract.address);
        expect(result.output.hours).to.equal(0);
    });

    it('Emits events.', async () => {
        const { contract, privi } = await setup();
        await privi.approve(contract.address, 10000);
        expect(contract.tx.subscribeN(contract.address, 1)).to.emit(contract, "SubscriptionEvent");
        expect(contract.tx.consume(contract.address, 50)).to.emit(contract, "ConsumeEvent");
    });

    it('Resets to 0 after end of month', async () => {
        const { contract, privi } = await setup({ subscriptionMaxAge: 0});
        await privi.approve(contract.address, 10000);
        await contract.tx.subscribeN(contract.address, 1);
        expect(contract.tx.consume(contract.address, 50)).to.not.emit(contract, "ConsumeEvent");
    });

    it('Only allows owner to consume hours correctly', async () => {
        const { contract, privi, Bob } = await setup();
        await privi.approve(contract.address, 10000);
        await contract.tx.subscribeN(contract.address, 1);
        expect(contract.connect(Bob).tx.consume(contract.address, 50)).to.not.emit(contract, "ConsumeEvent");
    });

    it('Allocates new hours if payed in advance', async () => {
        const { contract, privi } = await setup({ subscriptionMaxAge: 1});
        await privi.approve(contract.address, 10000);
        await contract.tx.subscribeN(contract.address, 2);
        expect(contract.tx.consume(contract.address, 50)).to.emit(contract, "ConsumeEvent");
        expect(contract.tx.consume(contract.address, 50)).to.emit(contract, "ConsumeEvent");
    });
});
