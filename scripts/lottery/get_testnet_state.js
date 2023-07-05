const { ApiPromise, WsProvider } = require("@polkadot/api");
const axios = require("axios");

async function main() {
  // Connect to the substrate node
  // const provider = new WsProvider('wss://crispy.baikal.testnet.calamari.systems:443');
  const provider = new WsProvider("ws://127.0.0.1:9111");
  const api = await ApiPromise.create({ provider });

  const lotteryAccount = "dmwQify2zjqgMQfDhDQRppgEeBFVvJaX3bq3xqtFmM94PKfgY";
  try {
    // const url = 'https://crispy.baikal.testnet.calamari.systems';
    const url = "http://127.0.0.1:9133";
    const request1 = {
      jsonrpc: "2.0",
      id: 1,
      method: "lottery_next_drawing_at",
      params: [],
    };
    const request2 = {
      jsonrpc: "2.0",
      id: 1,
      method: "lottery_current_prize_pool",
      params: [],
    };
    const request3 = {
      jsonrpc: "2.0",
      id: 1,
      method: "lottery_not_in_drawing_freezeout",
      params: [],
    };
    let next_drawing_block = await makeRequest(url, request1);
    let surplus_funds = await makeRequest(url, request2);
    let not_in_freezeout = await makeRequest(url, request3);

    // Funds in lottery palllet
    const block = (await api.query.system.number()).toJSON();
    console.log("block: \t\t\t\t\t \t", parseInt(block));
    const ltry_balances = (
      await api.query.system.account(lotteryAccount)
    ).toJSON().data;
    console.log("lotry-total: \t\t\t\t \t", parseInt(ltry_balances.free));
    let ltry_locked = (
      await api.query.balances.locks(lotteryAccount)
    ).toJSON()[0];
    ltry_locked == undefined
      ? (ltry_locked = 0)
      : (ltry_locked = ltry_locked.amount);
    console.log(
      "lotry-free: \t\t\t\t \t",
      parseInt(ltry_balances.free - ltry_locked)
    );
    const totalPot = await api.query.lottery.totalPot();
    console.log("lotry-totalpot:  \t\t\t\t", parseInt(totalPot));
    // console.log('totalpot-free:  \t\t\t\t', parseInt(totalPot)-parseInt(ltry_balances.free-ltry_locked));
    const reserve = (await api.query.lottery.gasReserve()).toString();
    console.log("lotry-reserve:  \t\t\t\t", reserve);
    const unclaimed = (
      await api.query.lottery.totalUnclaimedWinnings()
    ).toString();
    console.log("lotry-unclaimed:  \t\t\t\t", unclaimed);
    const unstaking = (
      await api.query.lottery.surplusUnstakingBalance()
    ).toString();
    console.log("lotry-unstaking:  \t\t\t\t", unstaking);
    console.log("lotry-surplus:  \t\t\t\t", surplus_funds);
    console.log(
      "free+unstaking+unclaimed:  \t\t",
      parseInt(ltry_balances.free - ltry_locked) +
        parseInt(unstaking) +
        parseInt(unclaimed)
    );
    console.log(
      "free+unstaking+unclaimed+pot:  \t",
      parseInt(ltry_balances.free - ltry_locked) +
        parseInt(unstaking) +
        parseInt(unclaimed) +
        parseInt(totalPot)
    );
    console.log("lotry-total: \t\t\t\t \t", parseInt(ltry_balances.free));

    // Staking
    const staked = await api.query.parachainStaking.delegatorState(
      lotteryAccount
    );
    if (staked.toJSON() == undefined) {
      console.log("staking-delegated:\t\t\t\t NONE");
    } else {
      console.log(
        "staking-delegated:\t\t\t\t",
        parseInt(staked.toJSON().total, 16)
      );
      console.log(
        "staking collators delegated to: \t" +
          staked.toJSON().delegations.length
      );
    }
    // const lotryCollators = await api.query.lottery.stakedCollators();
    // console.log("lotry collators delegated to: \t" + lotryCollators.toJSON().length);
    // for ((owner,amount) of staked.toJSON().delegations) {
    //     await api.query.parachainStaking.delegatorState(lotteryAccount);
    //     console.log('lotry-reserve:  \t', reserve);
    // }
  } catch (error) {
    console.error("Error:", error);
  } finally {
    // Disconnect from the substrate node
    await provider.disconnect();
  }
}

main().catch(console.error);

async function makeRequest(url, data) {
  try {
    const response = await axios.post(url, data, {
      headers: {
        "Content-Type": "application/json;charset=utf-8",
      },
    });
    // console.log(response.data.result);
    return response.data.result;
  } catch (error) {
    console.error(error);
  }
}
