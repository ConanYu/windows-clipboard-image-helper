import {invoke} from "@tauri-apps/api";
import {
  Button,
  Col,
  ColorPicker,
  DatePicker,
  Divider,
  Image as AntdImage,
  Input,
  Popover, Slider,
  Space,
  Spin,
  Switch,
} from "antd";
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
  const [coverRatio, setCoverRatio] = useState<[number, number]>([50, 100]);
  const [difference, setDifference] = useState(5);
  const [colorFilter, setColorFilter] = useState<[number, number, number] | undefined>(undefined);
  const showImage = (props: {
    reload: boolean,
    showImageSearchText?: string,
    showImageDateRange?: number[],
    showColorFilter?: [number, number, number],
    mtime?: number,
  }) => {
    const {reload, mtime} = props;
    let {showImageSearchText, showImageDateRange, showColorFilter} = props;
    setLoading(true);
    if (reload) {
      setPageNo(1);
      setImages([]);
    }
    showImageSearchText = showImageSearchText ?? searchText;
    showImageDateRange = showImageDateRange ?? dateRange;
    showColorFilter = showColorFilter ?? colorFilter;
    invoke('get_image', {
      request: {
        mtime,
        limit: 16,
        text: !!showImageSearchText ? [showImageSearchText] : undefined,
        date_range_from: showImageDateRange && showImageDateRange.length >= 1 ? showImageDateRange[0] * 1000 : undefined,
        date_range_to: showImageDateRange && showImageDateRange.length >= 2 ? showImageDateRange[1] * 1000 : undefined,
        color_filter: showColorFilter ? {
          red: showColorFilter[0],
          green: showColorFilter[1],
          blue: showColorFilter[2],
          cover_ratio_from: coverRatio[0] / 100.0,
          cover_ratio_to: coverRatio[1] / 100.0,
          difference: difference,
        } : undefined,
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
    showImage({reload: true});
  }, []);
  const content = [];
  let index = 0;
  let mtime: number | undefined = undefined;
  for (const image of images) {
    if (mtime === undefined || mtime > image.mtime) {
      mtime = image.mtime;
    }
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
              showImage({reload: false, mtime: mtime});
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
    return <Button type="primary" style={{marginLeft: 10}} disabled={loading} onClick={() => {
      showImage({reload: true});
    }}>查询</Button>;
  };
  return (
    <>
      <div style={{marginTop: 20, marginBottom: 10}}>
        <Input style={{marginLeft: 20, width: 600 + (moreCondition ? 80 : 0)}} addonBefore="图片文字" onChange={(e) => {
          setSearchText(e.target.value);
          showImage({reload: true, showImageSearchText: e.target.value});
        }}/>
        {moreCondition ? <></> : <SearchButton/>}
        <Switch style={{marginLeft: 10, position: "absolute", transform: 'translate(0, 25%)'}}
                checkedChildren="更多" unCheckedChildren="关闭" checked={moreCondition} onClick={(checked) => {
          setMoreCondition(checked);
          if (!checked) {
            setDateRange([]);
            setColorFilter(undefined);
            showImage({reload: true, showImageDateRange: []});
          }
        }}/>
        {
          moreCondition ? (
            <div style={{marginTop: 10, marginLeft: 20}}>
              <Space>
                <Col>
                  <RangePicker showTime onChange={(value) => {
                    if (value !== null && value.length >= 2) {
                      const v = value as any[];
                      const r = [v[0].unix(), v[1].unix()];
                      setDateRange(r);
                      showImage({reload: true, showImageDateRange: r});
                    } else {
                      setDateRange([]);
                      showImage({reload: true, showImageDateRange: []});
                    }
                  }}/>
                </Col>
                <Col>
                  <ColorPicker
                    onChange={(_, hex) => {
                      const r = parseInt(hex.substring(1, 3), 16);
                      const g = parseInt(hex.substring(3, 5), 16);
                      const b = parseInt(hex.substring(5, 7), 16);
                      setColorFilter([r, g, b]);
                    }}
                    onClear={() => {
                      setColorFilter(undefined);

                    }} allowClear defaultValue={null}
                    styles={{popupOverlayInner: {width: 468 + 24}}}
                    panelRender={(_, {components: {Picker}}) => (
                      <div className="custom-panel" style={{display: 'flex', width: 468}}>
                        <div style={{flex: 1}}>
                          <div>颜色覆盖率百分比范围</div>
                          <Slider range value={coverRatio} onChange={(value) => {
                            setCoverRatio(value);
                          }}/>
                          <div>可接受的DeltaE误差</div>
                          <Slider value={difference} onChange={(value) => {
                            setDifference(value);
                          }}/>
                          <div>使用颜色筛选可能比较慢</div>
                        </div>
                        <Divider type="vertical" style={{height: 'auto'}}/>
                        <div style={{width: 234}}><Picker/></div>
                      </div>
                    )}/>
                </Col>
              </Space>
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