import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/service/models.dart';
import 'package:esse/apps/assistant/page.dart';
import 'package:esse/apps/assistant/provider.dart';
import 'package:esse/apps/assistant/models.dart';
import 'package:esse/apps/file/page.dart';

const List<InnerService> INNER_SERVICES = [
  InnerService.Files,
  InnerService.Assistant,
];

class ServiceList extends StatefulWidget {
  const ServiceList({Key key}) : super(key: key);

  @override
  _ServiceListState createState() => _ServiceListState();
}

class _ServiceListState extends State<ServiceList> {
  @override
  Widget build(BuildContext context) {
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return Column(
      children: [
        Column(
          children: INNER_SERVICES.map((v) {
              final params = v.params(lang);
              return ListInnerService(
                name: params[0],
                bio: params[1],
                logo: params[2],
                callback: () => v.callback(context, isDesktop, lang),
                isDesktop: isDesktop,
              );
          }).toList()
        ),
        // Expanded(
        //   child: ListView.builder(
        //     itemCount: serviceKeys.length,
        //     itemBuilder: (BuildContext ctx, int index) => _ListService(),
        // )),
      ]
    );
  }
}

class ListInnerService extends StatelessWidget {
  final String name;
  final String bio;
  final String logo;
  final Function callback;
  final bool isDesktop;

  const ListInnerService({Key key,
      this.name, this.bio, this.logo, this.callback, this.isDesktop
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () {
        final widgets = this.callback();
        if (widgets != null) {
          if (this.isDesktop) {
            Provider.of<AccountProvider>(context, listen: false).updateActivedApp(widgets[0], widgets[1], widgets[2]);
          } else {
            if (widgets[2] != null) {
              Provider.of<AccountProvider>(context, listen: false).updateActivedApp(null, widgets[1], widgets[2]);
            } else {
              Navigator.push(context, MaterialPageRoute(builder: (_) => widgets[0]));
            }
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
                          child: Text(this.bio, maxLines: 1,
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
