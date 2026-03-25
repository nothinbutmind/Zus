// SPDX-License-Identifier: MIT
pragma solidity >=0.8.21;

interface IVerifier {
    function verify(bytes calldata proof, bytes32[] calldata publicInputs) external view returns (bool);
}

contract ZusProtocol {
    uint256 public constant PUBLIC_INPUTS_LENGTH = 74;
    uint256 private constant MESSAGE_LENGTH = 8;
    uint256 private constant ELIGIBLE_ROOT_INDEX = 8;
    uint256 private constant NULLIFIER_X_START = 9;
    uint256 private constant NULLIFIER_Y_START = 41;
    uint256 private constant STEALTH_ADDRESS_INDEX = 73;

    error NotOwner();
    error InvalidVerifier();
    error InvalidPayoutAmount();
    error InvalidPublicInputsLength(uint256 actualLength);
    error UnexpectedMessageByte(uint256 index, bytes32 actualWord, uint8 expectedByte);
    error UnexpectedEligibleRoot(bytes32 actualRoot, bytes32 expectedRoot);
    error UnexpectedPublicByte(uint256 index, bytes32 actualWord);
    error InvalidStealthAddress(bytes32 actualWord);
    error NullifierAlreadyUsed(bytes32 nullifierHash);
    error InvalidProof();
    error InsufficientBalance(uint256 available, uint256 required);
    error PayoutFailed();

    event Funded(address indexed funder, uint256 amount);
    event Claimed(
        address indexed caller, address indexed stealthRecipient, bytes32 indexed nullifierHash, uint256 payoutAmount
    );
    event Sweep(address indexed recipient, uint256 amount);

    struct ClaimPreview {
        bytes32 eligibleRoot;
        bytes32 nullifierHash;
        address stealthRecipient;
        bool alreadyClaimed;
        uint256 payoutAmount;
    }

    struct DecodedClaim {
        bytes32 eligibleRoot;
        bytes32 nullifierHash;
        address stealthRecipient;
    }

    IVerifier public immutable verifier;
    address public immutable owner;
    bytes32 public immutable eligibleRoot;
    bytes8 public immutable expectedMessage;
    uint256 public immutable payoutAmount;

    mapping(bytes32 => bool) public nullifierUsed;

    // MVP note: this deployment is a single drop with one root + one message domain.
    // The current circuit proves allowlist membership and the stealth recipient,
    // but it does not bind a per-leaf amount, so this contract pays a fixed amount per claim.
    constructor(address verifier_, bytes32 eligibleRoot_, bytes8 expectedMessage_, uint256 payoutAmount_) payable {
        if (verifier_ == address(0)) revert InvalidVerifier();
        if (payoutAmount_ == 0) revert InvalidPayoutAmount();

        verifier = IVerifier(verifier_);
        owner = msg.sender;
        eligibleRoot = eligibleRoot_;
        expectedMessage = expectedMessage_;
        payoutAmount = payoutAmount_;

        if (msg.value != 0) {
            emit Funded(msg.sender, msg.value);
        }
    }

    receive() external payable {
        emit Funded(msg.sender, msg.value);
    }

    function claim(bytes calldata proof, bytes32[] calldata publicInputs) external returns (address stealthRecipient) {
        DecodedClaim memory decoded = _decodeAndValidate(publicInputs);

        if (nullifierUsed[decoded.nullifierHash]) {
            revert NullifierAlreadyUsed(decoded.nullifierHash);
        }
        if (address(this).balance < payoutAmount) {
            revert InsufficientBalance(address(this).balance, payoutAmount);
        }

        bool verified = verifier.verify(proof, publicInputs);
        if (!verified) revert InvalidProof();

        nullifierUsed[decoded.nullifierHash] = true;
        stealthRecipient = decoded.stealthRecipient;

        (bool sent,) = stealthRecipient.call{value: payoutAmount}("");
        if (!sent) revert PayoutFailed();

        emit Claimed(msg.sender, stealthRecipient, decoded.nullifierHash, payoutAmount);
    }

    function previewClaim(bytes32[] calldata publicInputs) external view returns (ClaimPreview memory preview) {
        DecodedClaim memory decoded = _decodeAndValidate(publicInputs);

        preview = ClaimPreview({
            eligibleRoot: decoded.eligibleRoot,
            nullifierHash: decoded.nullifierHash,
            stealthRecipient: decoded.stealthRecipient,
            alreadyClaimed: nullifierUsed[decoded.nullifierHash],
            payoutAmount: payoutAmount
        });
    }

    function decodeStealthAddress(bytes32[] calldata publicInputs) external pure returns (address) {
        if (publicInputs.length != PUBLIC_INPUTS_LENGTH) {
            revert InvalidPublicInputsLength(publicInputs.length);
        }

        return _decodeStealthAddress(publicInputs[STEALTH_ADDRESS_INDEX]);
    }

    function sweep(address payable recipient, uint256 amount) external {
        if (msg.sender != owner) revert NotOwner();
        if (address(this).balance < amount) revert InsufficientBalance(address(this).balance, amount);

        (bool sent,) = recipient.call{value: amount}("");
        if (!sent) revert PayoutFailed();

        emit Sweep(recipient, amount);
    }

    function _decodeAndValidate(bytes32[] calldata publicInputs) internal view returns (DecodedClaim memory decoded) {
        if (publicInputs.length != PUBLIC_INPUTS_LENGTH) {
            revert InvalidPublicInputsLength(publicInputs.length);
        }

        _validateMessage(publicInputs);

        decoded.eligibleRoot = publicInputs[ELIGIBLE_ROOT_INDEX];
        if (decoded.eligibleRoot != eligibleRoot) {
            revert UnexpectedEligibleRoot(decoded.eligibleRoot, eligibleRoot);
        }

        bytes32 nullifierX = _packOutputBytes(publicInputs, NULLIFIER_X_START);
        bytes32 nullifierY = _packOutputBytes(publicInputs, NULLIFIER_Y_START);

        decoded.nullifierHash = keccak256(abi.encodePacked(nullifierX, nullifierY));
        decoded.stealthRecipient = _decodeStealthAddress(publicInputs[STEALTH_ADDRESS_INDEX]);
    }

    function _validateMessage(bytes32[] calldata publicInputs) internal view {
        bytes memory messageBytes = abi.encodePacked(expectedMessage);

        for (uint256 i = 0; i < MESSAGE_LENGTH; ++i) {
            uint256 actualWord = uint256(publicInputs[i]);
            uint8 expectedByte = uint8(messageBytes[i]);

            if (actualWord != expectedByte) {
                revert UnexpectedMessageByte(i, publicInputs[i], expectedByte);
            }
        }
    }

    function _packOutputBytes(bytes32[] calldata publicInputs, uint256 startIndex)
        internal
        pure
        returns (bytes32 packedBytes)
    {
        uint256 packed;

        for (uint256 i = 0; i < 32; ++i) {
            uint256 word = uint256(publicInputs[startIndex + i]);
            if (word > type(uint8).max) {
                revert UnexpectedPublicByte(startIndex + i, publicInputs[startIndex + i]);
            }
            packed = (packed << 8) | word;
        }

        packedBytes = bytes32(packed);
    }

    function _decodeStealthAddress(bytes32 publicInputWord) internal pure returns (address) {
        uint256 rawAddress = uint256(publicInputWord);
        if (rawAddress > type(uint160).max) {
            revert InvalidStealthAddress(publicInputWord);
        }

        return address(uint160(rawAddress));
    }
}
