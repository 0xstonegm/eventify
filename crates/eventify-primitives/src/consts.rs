/*
 * event Transfer(address indexed _from, address indexed _to, uint256 _value)
 * event Approval(address indexed _owner, address indexed _spender, uint256 _value)
 */
/// https://eips.ethereum.org/EIPS/eip-20
pub const EVENTS_ETH_ERC20: [&str; 2] = [
    "Transfer(address,address,uint256)",
    "Approval(address,address,uint256)",
];

/*
 * event Transfer(address indexed _from, address indexed _to, uint256 indexed _tokenId)
 * event Approval(address indexed _owner, address indexed _approved, uint256 indexed _tokenId)
 * event ApprovalForAll(address indexed _owner, address indexed _operator, bool _approved)
 */
/// https://eips.ethereum.org/EIPS/eip-721
pub const EVENTS_ETH_ERC721: [&str; 3] = [
    "Transfer(address,address,uint256)",
    "Approval(address,address,uint256)",
    "ApprovalForAll(address,address,bool)",
];

/*
 * event Sent(address indexed operator, address indexed from, address indexed to, uint256 amount, bytes data, bytes operatorData)
 * event Minted(address indexed operator, address indexed to, uint256 amount, bytes data, bytes operatorData)
 * event Burned(address indexed operator, address indexed from, uint256 amount, bytes data, bytes operatorData)
 * event AuthorizedOperator(address indexed operator,address indexed holder)
 * event RevokedOperator(address indexed operator, address indexed holder)
 */
/// https://eips.ethereum.org/EIPS/eip-777
pub const EVENTS_ETH_ERC777: [&str; 5] = [
    "Sent(address,address,address,uint256,bytes,bytes)",
    "Minted(address,address,uint256,bytes,bytes)",
    "Burned(address,address,uint256,bytes,bytes)",
    "AuthorizedOperator(address,address)",
    "RevokedOperator(address,address)",
];

/*
 * event TransferSingle(address indexed _operator, address indexed _from, address indexed _to, uint256 _id, uint256 _value)
 * event TransferBatch(address indexed _operator, address indexed _from, address indexed _to, uint256[] _ids, uint256[] _values)
 * event ApprovalForAll(address indexed _owner, address indexed _operator, bool _approved)
 * event URI(string _value, uint256 indexed _id)
 */
/// https://eips.ethereum.org/EIPS/eip-1155
pub const EVENTS_ETH_ERC1155: [&str; 4] = [
    "TransferSingle(address,address,address,uint256,uint256)",
    "TransferBatch(address,address,address,uint256[],uint256[])",
    "ApprovalForAll(address,address,bool)",
    "URI(string,uint256)",
];

/*
 * event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares)
 * event Withdraw(address indexed sender, address indexed receiver, address indexed owner, uint256 assets, uint256 shares)
 */
/// https://eips.ethereum.org/EIPS/eip-4626
pub const EVENTS_ETH_ERC4626: [&str; 2] = [
    "Deposit(address,address,uint256,uint256)",
    "Withdraw(address,address,address,uint256,uint256)",
];

///
pub const TRANSFER: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
pub const APPROVAL: &str = "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";
pub const APPROVAL_FOR_ALL: &str =
    "0x17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31";
pub const SENT: &str = "0x06b541ddaa720db2b10a4d0cdac39b8d360425fc073085fac19bc82614677987";
pub const MINTED: &str = "0x2fe5be0146f74c5bce36c0b80911af6c7d86ff27e89d5cfa61fc681327954e5d";
pub const BURNED: &str = "0xa78a9be3a7b862d26933ad85fb11d80ef66b8f972d7cbba06621d583943a4098";
pub const AUTHORIZED_OPERATOR: &str =
    "0xf4caeb2d6ca8932a215a353d0703c326ec2d81fc68170f320eb2ab49e9df61f9";
pub const REVOKED_OPERATOR: &str =
    "0x50546e66e5f44d728365dc3908c63bc5cfeeab470722c1677e3073a6ac294aa1";
pub const TRANSFER_SINGLE: &str =
    "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62";
pub const TRANSFER_BATCH: &str =
    "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb";
pub const URI: &str = "0x6bb7ff708619ba0610cba295a58592e0451dee2622938c8755667688daf3529b";
pub const DEPOSIT: &str = "0xdcbc1c05240f31ff3ad067ef1ee35ce4997762752e3a095284754544f4c709d7";
pub const WITHDRAW: &str = "0xfbde797d201c681b91056529119e0b02407c7bb96a4a2c75c01fc9667232c8db";
