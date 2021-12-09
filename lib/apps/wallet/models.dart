import 'package:flutter/material.dart';

enum ChainToken {
  ETH,
  ERC20,
  ERC721,
  BTC,
}

extension ChainTokenExtension on ChainToken {
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
        return ['Rinkeby Test Network', Colors.orange];
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

  Address.fromList(List params) {
    this.id = params[0];
    this.chain = ChainTokenExtension.fromInt(params[1]);
    this.index = params[2];
    this.name = params[3];
    this.address = params[4];
    this.isGen = params[5];
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
  String logo = 'assets/logo/logo_eth.png';

  Token() {}

  Token.fromList(List params) {
    this.id = params[0];
    this.chain = ChainTokenExtension.fromInt(params[1]);
    this.network = NetworkExtension.fromInt(params[2]);
    this.name = params[3];
    this.contract = params[4];
    this.decimal = params[5];
  }

  Token.eth(Network network) {
    this.network = network;
  }

  Token.btc(Network network) {
    this.network = network;
    this.name = 'BTC';
    this.decimal = 8;
  }

  balance(String number) {
    this.balanceString = number;

    final pad = number.length - (this.decimal + 1); // 0.00..00
    if (pad < 0) {
      number = ('0' * (-pad)) + number;
    }
    String right = number.substring(number.length - this.decimal, number.length);
    final left = number.substring(0, number.length - this.decimal);
    if (right.length > 8) {
      right = right.substring(0, 8);
    }
    final amount_s = left + '.' + right;
    this.amount = double.parse(amount_s);
  }
}
