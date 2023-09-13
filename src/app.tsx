import {useEffect, useState} from "react";
import {Button, FloatButton} from "antd";
import {AppstoreOutlined, ArrowLeftOutlined, SettingOutlined} from "@ant-design/icons";
import Index from "./index";
import Settings from "./settings";
import Detail from "./detail";
import {listen, TauriEvent} from "@tauri-apps/api/event";
import {invoke} from "@tauri-apps/api";

export default function App() {
  const [pageInfo, setPageInfo] = useState<'index' | 'settings' | 'detail' | 'empty'>('index');
  const [imageId, setImageId] = useState<number>(0);
  useEffect(() => {
    // 聚焦
    listen(TauriEvent.WINDOW_FOCUS, () => {
      invoke('get_escape_blur', {}).then((escape_blur) => {
        if (!escape_blur) {
          window.location.reload();
        }
      }).catch((e) => {
        console.error(e);
      });
    }).then(() => {
    });
  }, []);
  const Main = (() => {
    if (pageInfo === 'index') {
      return (
        <>
          <Index jumpDetailPage={(id) => {
            setPageInfo('detail');
            setImageId(id);
          }}/>
          <FloatButton icon={<SettingOutlined/>} style={{right: 24}} type="default" onClick={() => {
            setPageInfo('settings');
          }}></FloatButton>
        </>
      );
    } else if (pageInfo === 'settings') {
      return (
        <>
          <Settings/>
          <FloatButton icon={<AppstoreOutlined/>} type="default" style={{right: 36}} onClick={() => {
            setPageInfo('index');
          }}></FloatButton>
        </>
      );
    } else if (pageInfo === 'detail') {
      return (
        <>
          <div>
            <Button size="large" icon={<ArrowLeftOutlined/>} type="link" onClick={() => {
              setPageInfo('index');
            }}>返回</Button>
          </div>
          <Detail imageId={imageId} jumpIndex={() => {
            setPageInfo('index');
          }}/>
        </>
      );
    } else if (pageInfo === 'empty') {
      // 强制刷新使用
      return <></>;
    }
    const _: never = pageInfo;
    return <></>;
  })();
  return (
    <>
      {Main}
    </>
  );
}