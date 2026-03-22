// SPDX-License-Identifier: MIT
pragma solidity >=0.8.21;

import "../src/ZusProtocol.sol";

interface Vm {
    function deal(address account, uint256 newBalance) external;
    function prank(address sender) external;
    function expectRevert(bytes calldata revertData) external;
}

address constant HEVM_ADDRESS = address(uint160(uint256(keccak256("hevm cheat code"))));
Vm constant vm = Vm(HEVM_ADDRESS);

contract MockVerifier is IVerifier {
    bool public shouldVerify = true;

    function setShouldVerify(bool newValue) external {
        shouldVerify = newValue;
    }

    function verify(bytes calldata, bytes32[] calldata) external view returns (bool) {
        return shouldVerify;
    }
}

contract ZusProtocolTest {
    bytes8 internal constant MESSAGE = "ZUSMVP01";
    bytes32 internal constant ROOT = bytes32(uint256(0x1234));
    uint256 internal constant PAYOUT = 0.25 ether;

    MockVerifier internal verifier;
    ZusProtocol internal protocol;

    function setUp() public {
        verifier = new MockVerifier();
        protocol = new ZusProtocol{value: 1 ether}(address(verifier), ROOT, MESSAGE, PAYOUT);
    }

    function testPreviewClaimDecodesClaimData() public view {
        bytes32[] memory publicInputs = _buildPublicInputs(address(0xBEEF));
        bytes32 expectedNullifierHash = _expectedNullifierHash();

        ZusProtocol.ClaimPreview memory preview = protocol.previewClaim(publicInputs);

        require(preview.eligibleRoot == ROOT, "wrong root");
        require(preview.nullifierHash == expectedNullifierHash, "wrong nullifier");
        require(preview.stealthRecipient == address(0xBEEF), "wrong stealth recipient");
        require(!preview.alreadyClaimed, "unexpected claimed state");
        require(preview.payoutAmount == PAYOUT, "wrong payout amount");
    }

    function testDecodeStealthAddressReturnsRecipient() public view {
        bytes32[] memory publicInputs = _buildPublicInputs(address(0xCAFE));
        address stealthRecipient = protocol.decodeStealthAddress(publicInputs);

        require(stealthRecipient == address(0xCAFE), "wrong stealth recipient");
    }

    function testClaimPaysStealthAddressAndMarksNullifierUsed() public {
        address claimer = address(0x1111);
        address stealthRecipient = address(0xCAFE);
        bytes32[] memory publicInputs = _buildPublicInputs(stealthRecipient);
        bytes32 expectedNullifierHash = _expectedNullifierHash();
        uint256 beforeBalance = stealthRecipient.balance;

        vm.prank(claimer);
        address returnedRecipient = protocol.claim(hex"1234", publicInputs);

        require(returnedRecipient == stealthRecipient, "wrong return recipient");
        require(stealthRecipient.balance == beforeBalance + PAYOUT, "recipient not paid");
        require(protocol.nullifierUsed(expectedNullifierHash), "nullifier not marked");
    }

    function testClaimRevertsWhenVerifierRejects() public {
        verifier.setShouldVerify(false);
        bytes32[] memory publicInputs = _buildPublicInputs(address(0xCAFE));

        vm.expectRevert(abi.encodeWithSelector(ZusProtocol.InvalidProof.selector));
        protocol.claim(hex"1234", publicInputs);
    }

    function testClaimRevertsOnSecondUse() public {
        address stealthRecipient = address(0xCAFE);
        bytes32[] memory publicInputs = _buildPublicInputs(stealthRecipient);
        bytes32 expectedNullifierHash = _expectedNullifierHash();

        protocol.claim(hex"1234", publicInputs);

        require(protocol.nullifierUsed(expectedNullifierHash), "nullifier missing after first claim");

        vm.expectRevert(
            abi.encodeWithSelector(ZusProtocol.NullifierAlreadyUsed.selector, expectedNullifierHash)
        );
        protocol.claim(hex"1234", publicInputs);
    }

    function _buildPublicInputs(address stealthRecipient) internal pure returns (bytes32[] memory inputs) {
        inputs = new bytes32[](74);

        bytes memory messageBytes = abi.encodePacked(MESSAGE);
        for (uint256 i = 0; i < 8; ++i) {
            inputs[i] = bytes32(uint256(uint8(messageBytes[i])));
        }

        inputs[8] = ROOT;

        bytes memory nullifierX = abi.encodePacked(_nullifierX());
        bytes memory nullifierY = abi.encodePacked(_nullifierY());

        for (uint256 i = 0; i < 32; ++i) {
            inputs[9 + i] = bytes32(uint256(uint8(nullifierX[i])));
            inputs[41 + i] = bytes32(uint256(uint8(nullifierY[i])));
        }

        inputs[73] = bytes32(uint256(uint160(stealthRecipient)));
    }

    function _expectedNullifierHash() internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(_nullifierX(), _nullifierY()));
    }

    function _nullifierX() internal pure returns (bytes32) {
        return hex"0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
    }

    function _nullifierY() internal pure returns (bytes32) {
        return hex"2122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f40";
    }
}
