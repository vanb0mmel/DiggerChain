// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;

import {DiggerChainRewards, IExpeditionVerifier} from "../src/DiggerChainRewards.sol";

interface Vm {
    function warp(uint256 timestamp) external;
    function prank(address sender) external;
    function expectRevert(bytes4 selector) external;
}

// Workflow tests isolate accounting with a mock. Generated-verifier tests and
// the user-run testnet flow separately check real cryptographic proofs.
contract MockVerifier is IExpeditionVerifier {
    bool public valid = true;

    function setValid(bool value) external {
        valid = value;
    }

    function verify(uint256[8] calldata, uint256[14] calldata) external view returns (bool) {
        return valid;
    }
}

contract RewardsTest {
    Vm private constant vm = Vm(address(uint160(uint256(keccak256("hevm cheat code")))));
    address private constant ALICE = address(0xA11CE);
    address private constant BOB = address(0xB0B);
    MockVerifier private verifier;
    DiggerChainRewards private rewards;
    uint64 private start;

    function setUp() public {
        verifier = new MockVerifier();
        start = uint64(block.timestamp + 10);
        rewards = new DiggerChainRewards(address(verifier), 10, 60, 3, 1000, "DiggerChain Test", "TEST");
        vm.warp(start);
    }

    function inputs(address player, uint256 id, uint256 score, uint256 epoch)
        private
        view
        returns (uint256[14] memory value)
    {
        value[0] = 2;
        value[2] = block.chainid;
        value[3] = epoch;
        value[5] = score;
        value[10] = 20;
        value[11] = id;
        value[12] = uint256(uint160(player));
        value[13] = uint256(uint160(address(rewards)));
    }

    function register(address player, uint256 id, uint256 score, uint256 epoch) private {
        uint256[8] memory proof;
        uint256[14] memory value = inputs(player, id, score, epoch);
        vm.prank(player);
        rewards.register(proof, value);
    }

    function testProportionalPayoutsNeedNoFinalizer() public {
        register(ALICE, 1, 1, 0);
        register(BOB, 2, 3, 0);
        register(ALICE, 3, 2, 0);
        assert(rewards.totalScore(0) == 6);
        vm.expectRevert(DiggerChainRewards.ClaimUnavailable.selector);
        vm.prank(ALICE);
        rewards.claim(0);
        vm.warp(start + 60);
        assert(rewards.claimable(0, ALICE) == 500);
        vm.prank(ALICE);
        rewards.claim(0);
        vm.prank(BOB);
        rewards.claim(0);
        assert(rewards.balanceOf(ALICE) == 500 && rewards.balanceOf(BOB) == 500);
        assert(rewards.totalSupply() == 3000 && rewards.balanceOf(address(rewards)) == 2000);
        vm.expectRevert(DiggerChainRewards.ClaimUnavailable.selector);
        vm.prank(ALICE);
        rewards.claim(0);
    }

    function testDuplicateExpeditionIsRejectedAcrossPlayersAndEpochs() public {
        register(ALICE, 1, 3, 0);
        vm.expectRevert(DiggerChainRewards.ExpeditionAlreadyUsed.selector);
        register(BOB, 1, 4, 0);
        vm.warp(start + 60);
        vm.expectRevert(DiggerChainRewards.ExpeditionAlreadyUsed.selector);
        register(ALICE, 1, 3, 1);
    }

    function testWrongRecipientDomainRulesAndEpochAreRejected() public {
        uint256[8] memory proof;
        uint256[14] memory value = inputs(ALICE, 1, 3, 0);
        vm.expectRevert(DiggerChainRewards.WrongRecipient.selector);
        vm.prank(BOB);
        rewards.register(proof, value);
        for (uint256 i = 0; i < 3; ++i) {
            uint256 index = i == 0 ? 0 : (i == 1 ? 2 : 13);
            ++value[index];
            vm.expectRevert(DiggerChainRewards.InvalidDomain.selector);
            vm.prank(ALICE);
            rewards.register(proof, value);
            --value[index];
        }
        value[3] = 1;
        vm.expectRevert(DiggerChainRewards.EpochClosed.selector);
        vm.prank(ALICE);
        rewards.register(proof, value);
        value[3] = 0;
        vm.warp(start + 60);
        vm.expectRevert(DiggerChainRewards.EpochClosed.selector);
        vm.prank(ALICE);
        rewards.register(proof, value);
    }

    function testInvalidProofDoesNotConsumeAnExpedition() public {
        verifier.setValid(false);
        vm.expectRevert(DiggerChainRewards.InvalidProof.selector);
        register(ALICE, 1, 3, 0);
        assert(!rewards.usedExpeditions(bytes32(uint256(1))));
        assert(rewards.totalScore(0) == 0);
        verifier.setValid(true);
        register(ALICE, 1, 3, 0);
    }

    function testEmptyEpochAndRoundingCannotIncreaseIssuance() public {
        register(ALICE, 1, 1, 0);
        register(BOB, 2, 2, 0);
        vm.warp(start + 60);
        vm.prank(ALICE);
        rewards.claim(0);
        vm.prank(BOB);
        rewards.claim(0);
        assert(rewards.balanceOf(ALICE) == 333 && rewards.balanceOf(BOB) == 666);
        vm.warp(start + 120);
        assert(rewards.claimable(1, ALICE) == 0);
        vm.expectRevert(DiggerChainRewards.ClaimUnavailable.selector);
        vm.prank(ALICE);
        rewards.claim(1);
        assert(rewards.balanceOf(address(rewards)) == 2001);
    }

    function testNoncanonicalIdentityAndOutOfRangeResultsAreRejected() public {
        uint256[8] memory proof;
        uint256[14] memory value = inputs(ALICE, 1, 3, 0);
        value[4] = uint256(1) << 128;
        vm.expectRevert(DiggerChainRewards.InvalidResult.selector);
        vm.prank(ALICE);
        rewards.register(proof, value);
        value[4] = 0;
        value[11] = uint256(1) << 128;
        vm.expectRevert(DiggerChainRewards.InvalidResult.selector);
        vm.prank(ALICE);
        rewards.register(proof, value);
        value[11] = 1;
        value[5] = 64;
        vm.expectRevert(DiggerChainRewards.InvalidResult.selector);
        vm.prank(ALICE);
        rewards.register(proof, value);
        value[5] = 0;
        vm.expectRevert(DiggerChainRewards.InvalidResult.selector);
        vm.prank(ALICE);
        rewards.register(proof, value);
        value[5] = 3;
        value[10] = 513;
        vm.expectRevert(DiggerChainRewards.InvalidResult.selector);
        vm.prank(ALICE);
        rewards.register(proof, value);
    }

    function testTokenTransfersAndInvalidConfiguration() public {
        register(ALICE, 1, 3, 0);
        vm.warp(start + 60);
        vm.prank(ALICE);
        rewards.claim(0);
        vm.prank(ALICE);
        assert(rewards.transfer(BOB, 200));
        assert(rewards.balanceOf(BOB) == 200);
        vm.expectRevert(DiggerChainRewards.InvalidConfiguration.selector);
        new DiggerChainRewards(address(0), start, 60, 3, 1000, "Test", "TEST");
    }
}
