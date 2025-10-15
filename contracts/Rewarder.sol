// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title Rewarder
 * @dev Mining pool contract that rewards addresses based on CREATE2 hash difficulty
 */
contract Rewarder is ERC20, Ownable {
    struct MinerData {
        uint256 nonce;
        address minerAddress;
    }

    uint256 public constant INITIAL_SUPPLY = 1000000 * 10 ** 18; // 1 million tokens
    uint256 public constant REWARD_AMOUNT = 100 * 10 ** 18; // 100 tokens per successful mining
    address public difficulty;
    bytes32 public initHash =
        0x000000000000000000000000000000000000000000000000000000000000000000;

    // Events
    event DifficultyUpdated(address oldDifficulty, address newDifficulty);
    event RewardsDistributed(address indexed miner, uint256 totalReward);
    event MiningAttempt(address indexed miner, bool success, uint256 nonce);

    constructor(
        address _difficulty
    ) ERC20("Fonketh", "FONK") Ownable(msg.sender) {
        difficulty = _difficulty;
        _mint(address(this), INITIAL_SUPPLY);
    }

    /**
     * @dev Set the mining difficulty (minimum hash value required)
     * @param _difficulty The new difficulty value
     */
    function setDifficulty(address _difficulty) external onlyOwner {
        address oldDifficulty = difficulty;
        difficulty = _difficulty;
        emit DifficultyUpdated(oldDifficulty, _difficulty);
    }

    /**
     * @dev Set the mining difficulty (minimum hash value required)
     * @param _initHash The new difficulty value
     */
    function setInitHash(bytes32 _initHash) external onlyOwner {
        initHash = _initHash;
    }

    /** @dev Calculate the Nonce, MinerAddress Keccak256 hash */
    function sanityCheckNonce(
        uint256 nonce,
        address minerAddress
    ) public view returns (bytes32) {
        return keccak256(abi.encodePacked(nonce, minerAddress));
    }

    /**
     * @dev Calculate CREATE2 address for a given nonce and address
     * @param nonce The nonce value
     * @param minerAddress The miner's address
     * @return The calculated address
     */
    function calculateCreate2Address(
        uint256 nonce,
        address minerAddress
    ) public view returns (address) {
        // CREATE2 hash calculation: keccak256(0xff + deployer + salt + keccak256(bytecode))
        // For this implementation, we'll use a simplified version that combines nonce and address
        bytes32 salt = sanityCheckNonce(nonce, minerAddress);

        bytes32 hash = keccak256(
            abi.encodePacked(bytes1(0xff), address(this), salt, initHash)
        );

        // Convert the last 20 bytes of the hash to an address
        return address(uint160(uint256(hash)));
    }

    /**
     * @dev Check if an address meets the difficulty requirement
     * @param addr The address to check
     * @return True if address meets difficulty, false otherwise
     */
    function meetsDifficulty(address addr) public view returns (bool) {
        return addr < difficulty;
    }

    /**
     * @dev Process an array of 10 miners and distribute rewards
     * @param miners Array of MinerData containing nonce and address for each miner
     */
    function processMiningArray(MinerData[10] calldata miners) external {
        require(miners.length == 10, "Array must contain exactly 10 miners");

        // Process each miner
        for (uint256 i = 0; i < 10; i++) {
            MinerData memory miner = miners[i];

            // Calculate CREATE2 address
            address calculatedAddress = calculateCreate2Address(
                miner.nonce,
                miner.minerAddress
            );

            // Check if address meets difficulty
            bool success = meetsDifficulty(calculatedAddress);

            emit MiningAttempt(miner.minerAddress, success, miner.nonce);
            require(success, "Mining failed");
            ERC20(address(this)).transfer(miner.minerAddress, REWARD_AMOUNT);

            // Distribute rewards to successful miners
            emit RewardsDistributed(miner.minerAddress, REWARD_AMOUNT);
        }
    }

    /**
     * @dev Check if a specific nonce and address combination would be successful
     * @param nonce The nonce to check
     * @param minerAddress The address to check
     * @return True if the combination would be successful
     */
    function checkMiningSuccess(
        uint256 nonce,
        address minerAddress
    ) external view returns (bool) {
        address calculatedAddress = calculateCreate2Address(
            nonce,
            minerAddress
        );
        return meetsDifficulty(calculatedAddress);
    }
}
