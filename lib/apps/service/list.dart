import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/service/models.dart';
import 'package:esse/apps/service/add.dart';

const List<InnerService> INNER_SERVICES = [
  InnerService.Files,
  InnerService.Assistant,
  InnerService.GroupChat,
];

class ServiceList extends StatefulWidget {
  const ServiceList({Key? key}) : super(key: key);

  @override
  _ServiceListState createState() => _ServiceListState();
}

class _ServiceListState extends State<ServiceList> {
  @override
  Widget build(BuildContext context) {
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.services),
        bottom: PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.symmetric(vertical: 10.0),
        child: ListView.builder(
          itemCount: INNER_SERVICES.length,
          itemBuilder: (BuildContext ctx, int index) {
            final params = INNER_SERVICES[index].params(lang);
            return ListInnerService(
              name: params[0],
              bio: params[1],
              logo: params[2],
              callback: () => INNER_SERVICES[index].callback(),
              isDesktop: isDesktop,
            );
          }
        )
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          final widget = ServiceAddPage();
          if (isDesktop) {
            Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(widget);
          } else {
            Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
          }
        },
        child: const Icon(Icons.add, color: Colors.white),
        backgroundColor: Color(0xFF6174FF),
      ),
    );
  }
}

class ListInnerService extends StatelessWidget {
  final String name;
  final String? bio;
  final String logo;
  final Function callback;
  final bool isDesktop;

  const ListInnerService({Key? key,
      required this.name, this.bio, required this.logo, required this.callback, required this.isDesktop
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () {
        final widget = this.callback();
        if (widget != null) {
          if (this.isDesktop) {
            Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(widget);
          } else {
            Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
          }
        }
      },
      child: Container(
        height: 55.0,
        child: Row(
          children: [
            Container(
              width: 45.0,
              height: 45.0,
              padding: const EdgeInsets.all(6.0),
              margin: const EdgeInsets.only(left: 20.0, right: 15.0),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(15.0),
              ),
              child: Image.asset(this.logo),
            ),
            this.bio == null
            ? Text(this.name, style: TextStyle(fontSize: 16.0))
            : Expanded(
              child: Container(
                height: 55.0,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Expanded(
                          child: Text(this.name, maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(fontSize: 16.0)))
                      ]
                    ),
                    const SizedBox(height: 4.0),
                    Row(
                      children: [
                        Expanded(
                          child: Text(this.bio!, maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(color: Color(0xFFADB0BB), fontSize: 12.0)),
                        ),
                      ]
                    )
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
