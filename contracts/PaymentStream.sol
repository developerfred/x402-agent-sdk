// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract PaymentStream is ReentrancyGuard {
    
    IERC20 public immutable paymentToken;
    
    struct Stream {
        address sender;
        address recipient;
        uint256 totalAmount;
        uint256 withdrawnAmount;
        uint256 startTime;
        uint256 duration;
        bool isActive;
    }
    
    mapping(bytes32 => Stream) public streams;
    
    event StreamCreated(bytes32 indexed streamId, address indexed sender, uint256 amount);
    event StreamWithdrawal(bytes32 indexed streamId, uint256 amount);
    
    constructor(address _paymentToken) {
        require(_paymentToken != address(0), "Invalid token");
        paymentToken = IERC20(_paymentToken);
    }
    
    function createStream(
        address recipient,
        uint256 totalAmount,
        uint256 durationSeconds,
        bytes32 streamId
    ) external nonReentrant {
        require(recipient != address(0), "Invalid recipient");
        require(totalAmount > 0, "Amount must be > 0");
        require(!streams[streamId].isActive, "Stream ID exists");
        
        require(
            paymentToken.transferFrom(msg.sender, address(this), totalAmount),
            "Transfer failed"
        );
        
        streams[streamId] = Stream({
            sender: msg.sender,
            recipient: recipient,
            totalAmount: totalAmount,
            withdrawnAmount: 0,
            startTime: block.timestamp,
            duration: durationSeconds,
            isActive: true
        });
        
        emit StreamCreated(streamId, msg.sender, totalAmount);
    }
    
    function withdrawFromStream(bytes32 streamId) external nonReentrant {
        Stream storage stream = streams[streamId];
        
        require(stream.isActive, "Stream not active");
        require(stream.recipient == msg.sender, "Not recipient");
        
        uint256 withdrawable = getWithdrawableAmount(streamId);
        require(withdrawable > 0, "Nothing to withdraw");
        
        stream.withdrawnAmount += withdrawable;
        
        require(
            paymentToken.transfer(msg.sender, withdrawable),
            "Transfer failed"
        );
        
        emit StreamWithdrawal(streamId, withdrawable);
    }
    
    function getWithdrawableAmount(bytes32 streamId) public view returns (uint256) {
        Stream storage stream = streams[streamId];
        if (!stream.isActive) return 0;
        
        uint256 elapsed = block.timestamp - stream.startTime;
        uint256 maxWithdrawable = (stream.totalAmount * elapsed) / stream.duration;
        
        return maxWithdrawable - stream.withdrawnAmount;
    }
}
