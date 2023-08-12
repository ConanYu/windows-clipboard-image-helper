import {invoke} from "@tauri-apps/api";
import {Button, DatePicker, Image as AntdImage, Input, Popover, Spin, Switch} from "antd";
import React, {CSSProperties, useEffect, useState} from "react";
import {InView} from "react-intersection-observer";
import {CalcImagePaddleStyle, DateToString} from "./util";

const {RangePicker} = DatePicker;

function ImageBlock(props: { image: any, jumpDetailPage: (imageId: number) => void, onView?: (inView: boolean, entry: IntersectionObserverEntry) => void }) {
  const {image, jumpDetailPage, onView} = props;
  const {width, height, ctime, mtime} = image;
  const src = `data:image/png;base64,${image.image}`;
  const blockWidth = 170;
  const blockHeight = 170;
  const style = CalcImagePaddleStyle(blockWidth, blockHeight, width, height);
  const createDate = new Date(ctime);
  const modifyDate = new Date(mtime);
  return (
    <>
      {onView ? <InView as="span" onChange={onView}></InView> : <></>}
      <Popover content={
        <div>
          <div>添加时间：{DateToString(createDate)}</div>
          <div>上次使用：{DateToString(modifyDate)}</div>
        </div>
      }>
        <AntdImage width={blockWidth} height={blockHeight} preview={false} src={src} style={{
          ...style,
          cursor: "pointer",
        }} onClick={() => {
          jumpDetailPage(image.id);
        }}/>
      </Popover>
    </>
  );
}

export default function Index(props: { jumpDetailPage: (imageId: number) => void }) {
  const [loading, setLoading] = useState(false);
  const [moreCondition, setMoreCondition] = useState(false);
  const [images, setImages] = useState<any[]>([]);
  const [pageNo, setPageNo] = useState(1);
  const [lastImageLen, setLastImageLen] = useState(0);
  const [searchText, setSearchText] = useState('');
  const [dateRange, setDateRange] = useState<number[]>([]);
  const showImage = (reload: boolean, showImagePageNo: number, showImageSearchText?: string, dateRange?: number[]) => {
    setLoading(true);
    if (reload) {
      setImages([]);
    }
    invoke('get_image', {
      request: {
        page_no: showImagePageNo,
        page_size: 16,
        text: !!showImageSearchText ? [showImageSearchText] : undefined,
        date_range_from: dateRange && dateRange.length >= 1 ? dateRange[0] * 1000 : undefined,
        date_range_to: dateRange && dateRange.length >= 2 ? dateRange[1] * 1000 : undefined,
      }
    }).then((value) => {
      const v = value as any[];
      setLastImageLen(v.length);
      if (reload) {
        setImages(v as any[]);
      } else {
        setImages(images.concat(v));
      }
      setLoading(false);
    });
  };
  useEffect(() => {
    showImage(true, 1);
  }, []);
  const content = [];
  let index = 0;
  for (const image of images) {
    if (index % 4 === 0) {
      content.push(<div style={{marginTop: 20}} key={content.length}/>);
      content.push(<span style={{marginLeft: 15}} key={content.length}/>);
    } else {
      content.push(<span style={{marginLeft: 15}} key={content.length}/>);
    }
    content.push(
      <span key={content.length}>
        <ImageBlock image={image} jumpDetailPage={props.jumpDetailPage} onView={
          pageNo * 16 - 1 === index ? (inView) => {
            if (inView) {
              setPageNo(pageNo + 1);
              showImage(false, pageNo + 1, searchText, dateRange);
            }
          } : undefined
        }/>
      </span>
    );
    index += 1;
  }
  const footerStyle: CSSProperties = {
    textAlign: 'center',
    marginTop: 50,
    marginBottom: 30
  };
  const SearchButton = () => {
    return <Button type="primary" style={{marginLeft: 10}} onClick={() => {
      setPageNo(1);
      showImage(true, 1, searchText, dateRange);
    }}>查询</Button>;
  };
  return (
    <>
      <div style={{marginTop: 20, marginBottom: 10}}>
        <Input style={{marginLeft: 20, width: 600 + (moreCondition ? 80 : 0)}} addonBefore="图片文字" onChange={(e) => {
          setSearchText(e.target.value);
          showImage(true, 1, e.target.value);
        }}/>
        {moreCondition ? <></> : <SearchButton/>}
        <Switch style={{marginLeft: 10, position: "absolute", transform: 'translate(0, 25%)'}}
                checkedChildren="更多" unCheckedChildren="关闭" checked={moreCondition} onClick={(checked) => {
          setMoreCondition(checked);
        }}/>
        {
          moreCondition ? (
            <div style={{marginTop: 10, marginLeft: 20}}>
              <RangePicker showTime onChange={(value) => {
                if (value !== null && value.length >= 2) {
                  const v = value as any[];
                  const r = [v[0].unix(), v[1].unix()];
                  setDateRange(r);
                  showImage(true, 1, searchText, r);
                } else {
                  setDateRange([]);
                  showImage(true, 1, searchText, []);
                }
              }}/>
              <SearchButton/>
            </div>
          ) : <></>
        }
      </div>
      {content}
      {loading ? <div style={footerStyle}><Spin size="large"/></div> : <></>}
      {!loading && lastImageLen < 16 ? <div style={footerStyle}>已展示全部内容</div> : <></>}
    </>
  );
}