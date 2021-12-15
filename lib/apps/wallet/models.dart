import 'package:flutter/material.dart';

enum ChainToken {
  ETH,
  ERC20,
  ERC721,
  BTC,
}

extension ChainTokenExtension on ChainToken {
  bool isEth() {
    switch (this) {
      case ChainToken.ETH:
      case ChainToken.ERC20:
      case ChainToken.ERC721:
        return true;
      default:
        return false;
    }
  }

  bool isMain() {
    switch (this) {
      case ChainToken.ETH:
        return true;
      case ChainToken.ERC20:
      case ChainToken.ERC721:
        return false;
      case ChainToken.BTC:
        return true;
    }
  }

  String get symbol {
    switch (this) {
      case ChainToken.ETH:
      case ChainToken.ERC20:
      case ChainToken.ERC721:
        return 'ETH';
      case ChainToken.BTC:
        return 'BTC';
    }
  }

  int toInt() {
    switch (this) {
      case ChainToken.ETH:
        return 1;
      case ChainToken.ERC20:
        return 2;
      case ChainToken.ERC721:
        return 3;
      case ChainToken.BTC:
        return 4;
    }
  }

  static ChainToken fromInt(int a) {
    switch (a) {
      case 1:
        return ChainToken.ETH;
      case 2:
        return ChainToken.ERC20;
      case 3:
        return ChainToken.ERC721;
      case 4:
        return ChainToken.BTC;
      default:
        return ChainToken.ETH;
    }
  }
}

enum Network {
  EthMain,
  EthTestRopsten,
  EthTestRinkeby,
  EthTestKovan,
  EthLocal,
  BtcMain,
  BtcLocal,
}

extension NetworkExtension on Network {
  List params() {
    switch (this) {
      case Network.EthMain:
        return ['Ethereum Mainnet', Color(0xFF167F67)];
      case Network.EthTestRopsten:
        return ['Ropsten Test Network', Colors.orange];
      case Network.EthTestRinkeby:
        return ['Rinkeby Test Network', Colors.orange];
      case Network.EthTestKovan:
        return ['Kovan Test Network', Colors.orange];
      case Network.EthLocal:
        return ['Localhost 8545', Color(0xFF6174FF)];
      case Network.BtcMain:
        return ['Bitcoin Mainnet', Colors.purple];
      case Network.BtcLocal:
        return ['Localhost 8333', Color(0xFF6174FF)];
    }
  }

  String url() {
    switch (this) {
      case Network.EthMain:
        return 'https://etherscan.io/';
      case Network.EthTestRopsten:
        return 'https://ropsten.etherscan.io/';
      case Network.EthTestRinkeby:
        return 'https://rinkeby.etherscan.io/';
      case Network.EthTestKovan:
        return 'https://kovan.etherscan.io/';
      case Network.EthLocal:
        return 'https://etherscan.io/';
      case Network.BtcMain:
        return 'https://www.blockchain.com/btc/';
      case Network.BtcLocal:
        return 'https://www.blockchain.com/btc/';
    }
  }

  String txUrl() {
    return this.url() + '/tx/';
  }

  String tokenUrl() {
    return this.url() + '/token/';
  }

  int toInt() {
    switch (this) {
      case Network.EthMain:
        return 1;
      case Network.EthTestRopsten:
        return 2;
      case Network.EthTestRinkeby:
        return 3;
      case Network.EthTestKovan:
        return 4;
      case Network.EthLocal:
        return 5;
      case Network.BtcMain:
        return 6;
      case Network.BtcLocal:
        return 7;
    }
  }

  static Network fromInt(int a) {
    switch (a) {
      case 1:
        return Network.EthMain;
      case 2:
        return Network.EthTestRopsten;
      case 3:
        return Network.EthTestRinkeby;
      case 4:
        return Network.EthTestKovan;
      case 5:
        return Network.EthLocal;
      case 6:
        return Network.BtcMain;
      case 7:
        return Network.BtcLocal;
      default:
        return Network.EthMain;
    }
  }
}

class Address {
  int id = 0;
  ChainToken chain = ChainToken.ETH;
  int index = 0;
  String name = '';
  String address = '';
  bool isGen = true;
  bool isMain = false;
  Map<Network, String> balances = {};

  String icon() {
    return this.address.substring(2, 4);
  }

  List<Network> networks() {
    switch (this.chain) {
      case ChainToken.ETH:
      case ChainToken.ERC20:
      case ChainToken.ERC721:
        return [
          Network.EthMain,
          Network.EthTestRopsten,
          Network.EthTestRinkeby,
          Network.EthTestKovan,
          Network.EthLocal,
        ];
      case ChainToken.BTC:
        return [
          Network.BtcMain,
          Network.BtcLocal,
        ];
    }
  }

  String short() {
    final len = this.address.length;
    if (len > 10) {
      return this.address.substring(0, 6) + '...' + this.address.substring(len - 4, len);
    } else {
      return this.address;
    }
  }

  String balance(Network network) {
    if (this.balances.containsKey(network)) {
      final s = this.balances[network]!;
      switch (this.chain) {
        case ChainToken.ETH:
        case ChainToken.ERC20:
        case ChainToken.ERC721:
          return unitBalance(s, 18, 4);
        case ChainToken.BTC:
          return unitBalance(s, 8, 4);
      }
    } else {
      return '0.0';
    }
  }

  Token mainToken(Network network) {
    switch (this.chain) {
      case ChainToken.ETH:
      case ChainToken.ERC20:
      case ChainToken.ERC721:
        Token token = Token.eth(network);
        if (this.balances.containsKey(network)) {
          token.updateBalance(this.balances[network]!);
        }
        return token;
      case ChainToken.BTC:
        Token token = Token.btc(network);
        if (this.balances.containsKey(network)) {
          token.updateBalance(this.balances[network]!);
        }
        return token;
    }
  }

  split_balance(String s) {
    if (s.length > 0) {
      Map<Network, String> balances = {};
      s.split(",").forEach((ss) {
          final sss = ss.split(":");
          balances[NetworkExtension.fromInt(int.parse(sss[0]))] = sss[1];
      });

      this.balances = balances;
    }
  }

  Address.fromList(List params) {
    this.id = params[0];
    this.chain = ChainTokenExtension.fromInt(params[1]);
    this.index = params[2];
    this.name = params[3];
    this.address = params[4];
    this.isGen = params[5];
    this.isMain = params[6];
    this.split_balance(params[7]);
  }
}

class Token {
  int id = 0;
  ChainToken chain = ChainToken.ETH;
  Network network = Network.EthMain;
  String name = 'ETH';
  String contract = '';
  int decimal = 18;

  String balanceString = '';
  String balance = '0';
  double fiat = 0;

  Token() {}

  double get amount => double.parse(this.balance);

  String get logo {
    switch (name.toUpperCase()) {
      case 'ETH':
        return 'assets/logo/logo_eth.png';
      case 'USDT':
        return 'assets/logo/logo_tether.png';
      case 'ESNFT':
        return 'assets/logo/logo_esse_nft.png';
      default:
        if (chain == ChainToken.ERC20) {
          return 'assets/logo/logo_erc20.png';
        } else if (chain == ChainToken.ERC721) {
          return 'assets/logo/logo_nft.png';
        } else {
          return 'assets/logo/logo_btc.png';
        }
    }
  }

  bool isNft() {
    switch (this.chain) {
      case ChainToken.ERC721:
        return true;
      default:
        return false;
    }
  }

  String nftUrl(String hash) {
    return this.network.tokenUrl() + this.contract + '?a=' + hash;
  }

  Token.fromList(List params, String balance) {
    this.id = params[0];
    this.chain = ChainTokenExtension.fromInt(params[1]);
    this.network = NetworkExtension.fromInt(params[2]);
    this.name = params[3];
    this.contract = params[4];
    this.decimal = params[5];
    this.updateBalance(balance);
  }

  Token.eth(Network network) {
    this.network = network;
  }

  Token.btc(Network network) {
    this.network = network;
    this.name = 'BTC';
    this.decimal = 8;
  }

  String short() {
    final len = this.contract.length;
    if (len > 10) {
      return this.contract.substring(0, 6) + '...' + this.contract.substring(len - 4, len);
    } else {
      return this.contract;
    }
  }

  updateBalance(String number) {
    this.balanceString = number;

    if (number.length == 0 || number == '0') {
      this.balance = this.decimal > 0 ? '0.0' : '0';
    } else {
      this.balance = this.decimal == 0 ? number : unitBalance(number, this.decimal, 8);
    }
  }
}

class Transaction {
  String hash = '';
  String to = '';

  String short_hash() {
    final len = this.hash.length;
    if (len > 12) {
      return this.hash.substring(0, 8) + '...' + this.hash.substring(len - 4, len);
    } else {
      return this.hash;
    }
  }

  String short_to() {
    final len = this.to.length;
    if (len > 10) {
      return this.to.substring(0, 6) + '...' + this.to.substring(len - 4, len);
    } else {
      return this.to;
    }
  }

  Transaction.fromList(List params) {
    this.hash = params[0];
    this.to = params[1];
  }
}

String unitBalance(String number, int decimal, int limit) {
  final pad = number.length - (decimal + 1); // 0.00..00
  if (pad < 0) {
    number = ('0' * (-pad)) + number;
  }
  String right = number.substring(number.length - decimal, number.length);
  final left = number.substring(0, number.length - decimal);
  if (limit == 0) {
    return left;
  }

  if (right.length > limit) {
    right = right.substring(0, limit);
  }
  return left + '.' + right;
}

String restoreBalance(String number, int decimal) {
  if (decimal == 0) {
    return number;
  }

  final s = number.split('.');
  int pad = decimal;
  String right = '';

  if (s.length > 1 && s[1].length > 0) {
    if (s[1].length > decimal) {
      right = s[1].substring(0, decimal);
    } else {
      right = s[1] + ('0' * (decimal - s[1].length));
    }
  } else {
    right = '0' * decimal;
  }

  return s[0] + right;
}
