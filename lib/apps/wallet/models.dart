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
          return unit_balance(s, 18, 4);
        case ChainToken.BTC:
          return unit_balance(s, 8, 4);
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
          token.balance(this.balances[network]!);
        }
        return token;
      case ChainToken.BTC:
        Token token = Token.btc(network);
        if (this.balances.containsKey(network)) {
          token.balance(this.balances[network]!);
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
    this.split_balance(params[6]);
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
  double amount = 0.0;
  double fiat = 0.0;

  Token() {}

  String get logo {
    switch (name.toUpperCase()) {
      case 'ETH':
        return 'assets/logo/logo_eth.png';
      case 'USDT':
        return 'assets/logo/logo_tether.png';
      default:
        if (chain.isEth()) {
          return 'assets/logo/logo_erc20.png';
        } else {
          return 'assets/logo/logo_btc.png';
        }
    }
  }

  Token.fromList(List params, String balance) {
    this.id = params[0];
    this.chain = ChainTokenExtension.fromInt(params[1]);
    this.network = NetworkExtension.fromInt(params[2]);
    this.name = params[3];
    this.contract = params[4];
    this.decimal = params[5];
    this.balance(balance);
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

  balance(String number) {
    this.balanceString = number;
    this.amount = double.parse(unit_balance(number, this.decimal, 8));
  }
}

String unit_balance(String number, int decimal, int limit) {
  final pad = number.length - (decimal + 1); // 0.00..00
  if (pad < 0) {
    number = ('0' * (-pad)) + number;
  }
  String right = number.substring(number.length - decimal, number.length);
  final left = number.substring(0, number.length - decimal);
  if (right.length > limit) {
    right = right.substring(0, limit);
  }
  return left + '.' + right;
}
