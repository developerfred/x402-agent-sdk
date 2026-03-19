// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract SimplePaymentReceiver is ReentrancyGuard {
    
    IERC20 public immutable paymentToken;
    
    mapping(bytes32 => bool) public usedPaymentHashes;
    
    event PaymentReceived(
        address indexed from,
        uint256 amount,
        bytes32 paymentId,
        string metadata
    );
    
    constructor(address _paymentToken) {
        paymentToken = IERC20(_paymentToken);
    }
    
    function pay(
        uint256 amount,
        bytes32 paymentId,
        string calldata metadata
    ) external nonReentrant {
        require(amount > 0, "Amount must be > 0");
        
        require(
            paymentToken.transferFrom(msg.sender, address(this), amount),
            "Transfer failed"
        );
        
        bytes32 paymentHash = keccak256(abi.encodePacked(
            msg.sender,
            amount,
            paymentId,
            block.timestamp
        ));
        require(!usedPaymentHashes[paymentHash], "Payment already used");
        usedPaymentHashes[paymentHash] = true;
        
        emit PaymentReceived(msg.sender, amount, paymentId, metadata);
    }
    
    function verifyPayment(
        address payer,
        uint256 amount,
        bytes32 paymentId
    ) external view returns (bool) {
        bytes32 paymentHash = keccak256(abi.encodePacked(
            payer,
            amount,
            paymentId,
            block.timestamp
        ));
        return usedPaymentHashes[paymentHash];
    }
}
