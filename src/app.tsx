import {useState} from "react";
import {Button, FloatButton} from "antd";
import {AppstoreOutlined, ArrowLeftOutlined, SettingOutlined} from "@ant-design/icons";
import Index from "./index";
import Settings from "./settings";
import Detail from "./detail";

export default function App() {
  const [pageInfo, setPageInfo] = useState<'index' | 'settings' | 'detail'>('index');
  const [imageId, setImageId] = useState<number>(0);
  if (pageInfo === 'index') {
    return (
      <>
        <Index jumpDetailPage={(id) => {
          setPageInfo('detail');
          setImageId(id);
        }}/>
        <FloatButton icon={<SettingOutlined/>} type="default" style={{right: 24}} onClick={() => {
          setPageInfo('settings');
        }}></FloatButton>
      </>
    );
  } else if (pageInfo === 'settings') {
    return (
      <>
        <Settings/>
        <FloatButton icon={<AppstoreOutlined/>} type="default" style={{right: 24}} onClick={() => {
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
  }
  const _: never = pageInfo;
  return <></>;
}