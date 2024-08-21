import React from 'react';
import styled from '@emotion/styled';
import {Status,TLog} from '../../types';
import {RED,YELLOW,GREEN,BLUE,PURPLE} from '../../constants';

/**
 * 使用React.memo生成的子组件只要props的值不改变子组件不会被重新渲染
 */
const Log = React.memo((props: TLog) => (
    <Column>
        <Row>
            <StyledSpan status={props.status}>
                {'>'} {props.status}
            </StyledSpan>
            {props.method && <Method> [{props.method}]</Method>}
        </Row>
        <Message>{props.message}</Message>
        {props.messageTwo && <Message>{props.messageTwo}</Message>}
    </Column>
));

export default Log;

const Column = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  line-height: 1.5;
`;

const Row = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

const StyledSpan = styled.span<{ status: Status }>`
  color: ${(props) => {
    switch (props.status) {
        case 'success':
            return GREEN;
        case 'warning':
            return YELLOW;
        case 'error':
            return RED;
        case 'info':
            return BLUE;
    }
}};
  margin-right: 5px;
`;

const Method = styled.p`
  color: ${PURPLE};
  margin-right: 10px;
`;

const Message = styled.p`
  overflow-wrap: break-word;
`;