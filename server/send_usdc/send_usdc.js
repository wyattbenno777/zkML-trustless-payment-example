const { ethers } = require("ethers");
require("dotenv").config();

SEPOLIA_RPC_ENDPOINT = process.env.SEPOLIA_RPC_ENDPOINT;
SEPOLIA_USDC_CONTRACT = process.env.SEPOLIA_USDC_CONTRACT;
SENDER_PRIVATE_KEY = process.env.SENDER_PRIVATE_KEY;
SENDER_ADDRESS = process.env.SENDER_ADDRESS;

// 1. Input the Sepolia RPC endpoint
const rpcEndpoint = SEPOLIA_RPC_ENDPOINT; // Insert the INFURA Network Endpoint

// 2. Initialize the Ethers.js provider
const provider = new ethers.JsonRpcProvider(rpcEndpoint);

// 3. Input USDC token contract address for Ethereum Sepolia
const tokenAddress = SEPOLIA_USDC_CONTRACT; // USDC TokenAddress

// 4. Define the USDC token contract ABI
const minTokenAbi = [
  {
    inputs: [
      { internalType: "address", name: "to", type: "address" },
      { internalType: "uint256", name: "value", type: "uint256" },
    ],
    name: "transfer",
    outputs: [{ internalType: "bool", name: "", type: "bool" }],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ internalType: "address", name: "account", type: "address" }],
    name: "balanceOf",
    outputs: [{ internalType: "uint256", name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "decimals",
    outputs: [{ internalType: "uint8", name: "", type: "uint8" }],
    stateMutability: "view",
    type: "function",
  },
];

// 5. Create a new contract instance
const contract = new ethers.Contract(tokenAddress, minTokenAbi, provider);

// 6. Input the addresses and the private key; specify number of tokens to send
const senderAddress = SENDER_ADDRESS;
const recipientAddress = "0x73987bF167b5cC201cBa676F64d43A063C62018b";
const senderPrivateKey = SENDER_PRIVATE_KEY;

const usdcAmount = 1.0;

async function main() {
  // 7. Check the number of decimals for the USDC token
  const decimals = await contract.decimals();

  // 8. Check the balance of the sender's address
  const balance = await contract.balanceOf(senderAddress);
  console.log("Sender USDC balance:", ethers.formatUnits(balance, decimals));

  // 9. Calculate the actual amount in the smallest unit
  const value = ethers.parseUnits(usdcAmount.toString(), decimals);

  // 10. Create the transaction
  const tx = await contract
    .connect(new ethers.Wallet(senderPrivateKey, provider))
    .transfer(recipientAddress, value);

  // 11. Wait for the transaction to be mined and log the transaction hash
  const receipt = await tx.wait();
  console.log("Tx Hash:", receipt);
}

main().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});
