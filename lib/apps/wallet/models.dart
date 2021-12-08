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

class Address {
  int id = 0;
  ChainToken chain = ChainToken.ETH;
  int index = 0;
  String name = '';
  String address = '';
  bool isGen = true;

  Address.fromList(List params) {
    this.id = params[0];
    this.chain = ChainTokenExtension.fromInt(params[1]);
    this.index = params[2];
    this.name = params[3];
    this.address = params[4];
    this.isGen = params[5];
  }
}
