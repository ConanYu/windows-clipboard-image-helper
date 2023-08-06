import {useState} from "react";
import {FloatButton} from "antd";
import {AppstoreOutlined, SettingOutlined} from "@ant-design/icons";
import Index from "./index";
import Settings from "./settings";

export default function App() {
  const [pageInfo, setPageInfo] = useState<'index' | 'settings'>('index');
  if (pageInfo === 'index') {
    return (
      <>
        <Index/>
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
  }
  const _: never = pageInfo;
  return <></>;
}