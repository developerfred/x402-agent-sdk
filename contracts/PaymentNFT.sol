// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract PaymentNFT is ERC721, ERC721URIStorage, ReentrancyGuard {
    
    IERC20 public immutable paymentToken;
    uint256 public nextTokenId;
    uint256 public pricePerNFT;
    address public treasury;
    
    mapping(bytes32 => bool) public usedPaymentHashes;
    
    event NFTMinted(
        address indexed to,
        uint256 tokenId,
        uint256 amount,
        bytes32 paymentId
    );
    
    constructor(
        address _paymentToken,
        address _treasury,
        uint256 _pricePerNFT
    ) ERC721("Payment NFT", "PNFT") {
        paymentToken = IERC20(_paymentToken);
        treasury = _treasury;
        pricePerNFT = _pricePerNFT;
    }
    
    function mintWithPayment(
        bytes32 paymentId,
        string memory tokenURI
    ) external nonReentrant returns (uint256) {
        require(pricePerNFT > 0, "NFTs not for sale");
        
        require(
            paymentToken.transferFrom(msg.sender, treasury, pricePerNFT),
            "Payment failed"
        );
        
        bytes32 paymentHash = keccak256(abi.encodePacked(
            msg.sender,
            pricePerNFT,
            paymentId,
            block.timestamp
        ));
        require(!usedPaymentHashes[paymentHash], "Payment already used");
        usedPaymentHashes[paymentHash] = true;
        
        uint256 tokenId = nextTokenId++;
        _mint(msg.sender, tokenId);
        _setTokenURI(tokenId, tokenURI);
        
        emit NFTMinted(msg.sender, tokenId, pricePerNFT, paymentId);
        
        return tokenId;
    }
    
    function tokenURI(uint256 tokenId)
        public view override(ERC721, ERC721URIStorage)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }
}
