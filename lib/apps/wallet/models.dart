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
      case Network.EthLocal:
        return ['Localhost 8545', Color(0xFF6174FF)];
      case Network.BtcMain:
        return ['Bitcoin Mainnet', Colors.purple];
      case Network.BtcLocal:
        return ['Localhost 8333', Color(0xFF6174FF)];
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
