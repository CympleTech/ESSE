class ProviderServer {
  int id;
  String name;
  String addr;
  bool isOk;
  bool isDefault;
  bool isProxy;
  bool isActived;
  bool deletable = true;

  ProviderServer.empty():
    this.id = 0,
    this.name = '',
    this.addr = '',
    this.isOk = false,
    this.isDefault = false,
    this.isProxy = false,
    this.isActived = false;

  ProviderServer.fromList(List params):
    this.id = params[0],
    this.name = params[1],
    this.addr = params[2],
    this.isOk = params[3],
    this.isDefault = params[4],
    this.isProxy = params[5],
    this.isActived = params[6];
}

class Name {
  int id;
  int provider;
  String name;
  String bio;
  bool isOk;
  bool isActived;

  Name.fromList(List params):
    this.id = params[0],
    this.provider = params[1],
    this.name = params[2],
    this.bio = params[3],
    this.isOk = params[4],
    this.isActived = params[5];
}
