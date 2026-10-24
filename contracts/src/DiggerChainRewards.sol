// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;

import {ERC20} from "../vendor/openzeppelin/contracts/token/ERC20/ERC20.sol";

interface IExpeditionVerifier {
    function verify(uint256[8] calldata proof, uint256[14] calldata inputs) external view returns (bool);
}

contract DiggerChainRewards is ERC20 {
    uint256 public constant RULES_VERSION = 2;
    uint256 public constant MAX_SCORE = 63;
    uint256 public constant MAX_ACTIONS = 512;

    IExpeditionVerifier public immutable verifier;
    bytes32 public immutable verifierCodeHash;
    uint64 public immutable startsAt;
    uint64 public immutable epochDuration;
    uint64 public immutable epochCount;
    uint128 public immutable epochBudget;

    mapping(bytes32 expedition => bool) public usedExpeditions;
    mapping(uint256 epoch => uint128) public totalScore;
    mapping(uint256 epoch => mapping(address player => uint128)) public scoreOf;
    mapping(uint256 epoch => mapping(address player => bool)) public claimed;

    error InvalidConfiguration();
    error InvalidDomain();
    error WrongRecipient();
    error EpochClosed();
    error InvalidResult();
    error ExpeditionAlreadyUsed();
    error InvalidProof();
    error ClaimUnavailable();

    event ScoreRegistered(uint256 indexed epoch, address indexed player, bytes32 indexed expedition, uint256 score);
    event RewardClaimed(uint256 indexed epoch, address indexed player, uint256 amount);

    constructor(
        address verifier_,
        uint64 startDelay_,
        uint64 duration_,
        uint64 count_,
        uint128 budget_,
        string memory name_,
        string memory symbol_
    ) ERC20(name_, symbol_) {
        uint256 start = block.timestamp + startDelay_;
        if (
            verifier_.code.length == 0 || duration_ == 0 || count_ == 0 || budget_ == 0 || bytes(name_).length == 0
                || bytes(symbol_).length == 0 || start + uint256(duration_) * count_ > type(uint64).max
        ) {
            revert InvalidConfiguration();
        }
        verifier = IExpeditionVerifier(verifier_);
        verifierCodeHash = verifier_.codehash;
        startsAt = uint64(start);
        epochDuration = duration_;
        epochCount = count_;
        epochBudget = budget_;
        _mint(address(this), uint256(budget_) * count_);
    }

    function epochEnd(uint256 epoch) public view returns (uint256) {
        if (epoch >= epochCount) revert EpochClosed();
        return uint256(startsAt) + (epoch + 1) * epochDuration;
    }

    function currentEpoch() external view returns (uint256) {
        if (block.timestamp < startsAt) revert EpochClosed();
        uint256 epoch = (block.timestamp - startsAt) / epochDuration;
        if (epoch >= epochCount) revert EpochClosed();
        return epoch;
    }

    function register(uint256[8] calldata proof, uint256[14] calldata inputs) external {
        if (inputs[0] != RULES_VERSION || inputs[2] != block.chainid || inputs[13] != uint256(uint160(address(this)))) {
            revert InvalidDomain();
        }
        if (inputs[12] != uint256(uint160(msg.sender))) revert WrongRecipient();
        uint256 epoch = inputs[3];
        uint256 deadline = epochEnd(epoch);
        if (block.timestamp < deadline - epochDuration || block.timestamp >= deadline) revert EpochClosed();
        uint256 score = inputs[5];
        if (
            score == 0 || score > MAX_SCORE || inputs[10] > MAX_ACTIONS || inputs[4] > type(uint128).max
                || inputs[11] > type(uint128).max
        ) revert InvalidResult();
        bytes32 expedition = bytes32((inputs[4] << 128) | inputs[11]);
        if (usedExpeditions[expedition]) revert ExpeditionAlreadyUsed();
        if (!verifier.verify(proof, inputs)) revert InvalidProof();
        usedExpeditions[expedition] = true;
        totalScore[epoch] += uint128(score);
        scoreOf[epoch][msg.sender] += uint128(score);
        emit ScoreRegistered(epoch, msg.sender, expedition, score);
    }

    function claimable(uint256 epoch, address player) public view returns (uint256) {
        if (
            epoch >= epochCount || block.timestamp < epochEnd(epoch) || claimed[epoch][player] || totalScore[epoch] == 0
        ) return 0;
        return uint256(epochBudget) * scoreOf[epoch][player] / totalScore[epoch];
    }

    function claim(uint256 epoch) external returns (uint256 amount) {
        if (
            epoch >= epochCount || block.timestamp < epochEnd(epoch) || claimed[epoch][msg.sender]
                || scoreOf[epoch][msg.sender] == 0
        ) revert ClaimUnavailable();
        amount = claimable(epoch, msg.sender);
        claimed[epoch][msg.sender] = true;
        _transfer(address(this), msg.sender, amount);
        emit RewardClaimed(epoch, msg.sender, amount);
    }
}
