/* ── Adiyutant v1.1 — Today behaviour pass ──
   Six architectural fixes on top of v1.0:
   1. Active execution state lifted to App (survives Сегодня↔План switching);
      elapsed is derived from session.startTime + accumulatedSec, not stored.
   2. Stable taskId layer above blockId. Top 3, completion, DoSomethingElse,
      End-of-Day and "Не делал" all reference taskId.
   3. DoSomethingElse applies the chosen handling (leave/shift/move) against
      the actual schedule.
   4. End-of-Day moves the same logical task: removes the current block, keeps
      taskId, then either creates a new block on the target date (without a
      fictitious time) or returns to backlog. No clones.
   5. "Не делал" no longer records a factual session; offers
      Перенести / В бэклог / Оставить.
   6. Agent CTAs use a shared onNavigate() — setTab is no longer referenced
      from within TodayScreen.
   Data model: Task (stable taskId) → ScheduledBlock (blockId) → WorkSession (fact).
*/

const T={bg:'#0A121B',bgDeep:'#071019',surface:'#111A24',surface2:'#151F2A',
  surface3:'#172230',fg:'#F4F7FB',muted:'#B5BFCC',muted2:'#7D8997',
  border:'#253240',accent:'#2F80FF',accentSoft:'#183D77',accentDeep:'#0F2C5C',
  accentGreen:'#31D06D',accentGreenDeep:'#0F3B23',accentOrange:'#FFAD3D',
  accentOrangeDeep:'#3D2A0F',accentPurple:'#A660FF',accentPurpleDeep:'#26183D',
  accentRed:'#FF4D4D',accentRedDeep:'#3D1414',font:'"SF Pro Display","Inter",system-ui,sans-serif',
  mono:'ui-monospace,"SF Mono",Menlo,monospace'};

const TODAY='2026-08-14';
const TODAYS={
  d:'Пят',dLong:'14 августа',mm:'августа',num:14,monthIndex:7,
  iso:TODAY,getDate(){return new Date(this.iso+'T12:00:00');},
  addDays(n){const d=this.getDate();d.setDate(d.getDate()+n);return d.toISOString().slice(0,10);},
};
const NEXT_DAY=TODAYS.addDays(1);
TODAYS.tomorrow={iso:NEXT_DAY};

/* Each logical task carries a stable taskId (tN for scheduled, tuN for unscheduled).
   blockId (sN) is a placement in the schedule and may change on reschedule;
   taskId is permanent and is what Top 3, completion, projects, and the future
   Kotlin ViewModel refer to. */
const INIT_UNSCHEDULED=[
  {taskId:'tu1',title:'Linux namespaces & cgroups',project:'Обучение',duration:90,priority:'high',plannedDate:'2026-08-14'},
  {taskId:'tu2',title:'English speech practice',project:'Self Development',duration:45,priority:'medium',plannedDate:'2026-08-14'},
  {taskId:'tu3',title:'Позвонить в техподдержку',project:'Личное',duration:15,priority:'low',plannedDate:'2026-08-15'},
  {taskId:'tu4',title:'Обзор статьи по Kubernetes',project:'Обучение',duration:30,priority:'medium',plannedDate:null},
  {taskId:'tu5',title:'Написать тесты для формы',project:'Adiyutant',duration:60,priority:'high',plannedDate:'2026-08-16'},
  {taskId:'tu6',title:'Подготовить презентацию',project:'Adiyutant',duration:45,priority:'low',plannedDate:null,postponed:true},
];

const INIT_SCHEDULED={
  '2026-08-14':[
    {id:'s1',taskId:'t1',title:'Планирование дня',project:'Adiyutant',time:'09:00',end:'10:00',color:T.accent,progress:100},
    {id:'s2',taskId:'t2',title:'Код ревью PR #42',project:'Adiyutant',time:'10:15',end:'11:15',color:T.accent,progress:100},
    {id:'s3',taskId:'t3',title:'Deep work: React компоненты',project:'Adiyutant',time:'11:30',end:'13:00',color:T.accentPurple,progress:45},
    {id:'s4',taskId:'t4',title:'Sprint Planning',project:'Adiyutant',time:'14:00',end:'15:00',color:T.accent,progress:0},
    {id:'s5',taskId:'t5',title:'Дизайн-система: токены',project:'Adiyutant',time:'15:15',end:'16:35',color:T.accent,progress:0},
    {id:'s6',taskId:'t6',title:'Запрос на хостинг',project:'Инфраструктура',time:'17:00',end:'17:30',color:T.accentOrange,progress:0},
  ],
  '2026-08-15':[
    {id:'s7',taskId:'t7',title:'Ретроспектива спринта',project:'Adiyutant',time:'10:00',end:'11:00',color:T.accent,progress:0},
    {id:'s8',taskId:'t8',title:'Деплой v2.1',project:'Adiyutant',time:'13:00',end:'14:30',color:T.accentGreen,progress:0},
    {id:'s9',taskId:'t9',title:'Deep work: архитектура',project:'Adiyutant',time:'15:00',end:'16:30',color:T.accentPurple,progress:0},
  ],
  '2026-08-16':[
    {id:'s10',taskId:'t10',title:'Субботний хакатон',project:'Self Development',time:'10:00',end:'13:00',color:T.accentPurple,progress:0},
  ],
  '2026-08-13':[
    {id:'s11',taskId:'t11',title:'Onboarding встречи',project:'Adiyutant',time:'09:00',end:'10:30',color:T.accent,progress:100},
    {id:'s12',taskId:'t12',title:'Deep work: бэкенд',project:'Adiyutant',time:'11:00',end:'13:00',color:T.accentPurple,progress:100},
    {id:'s13',taskId:'t13',title:'Обед',project:'',time:'13:00',end:'14:00',color:'transparent',progress:100},
    {id:'s14',taskId:'t14',title:'Code review',project:'Adiyutant',time:'14:00',end:'14:45',color:T.accent,progress:100},
  ],
  '2026-08-12':[
    {id:'s15',taskId:'t15',title:'Дизайн-сессия',project:'Adiyutant',time:'10:00',end:'12:00',color:T.accentPurple,progress:100},
    {id:'s16',taskId:'t16',title:'Интеграционные тесты',project:'Adiyutant',time:'13:00',end:'15:00',color:T.accent,progress:100},
  ],
};

/* ── helpers ── */
const WEEKDAYS_SHORT=['Вс','Пн','Вт','Ср','Чт','Пт','Сб'];
const MONTHS=['янв','фев','мар','апр','мая','июн','июл','авг','сен','окт','ноя','дек'];
const MONTHS_LONG=['января','февраля','марта','апреля','мая','июня','июля','августа','сентября','октября','ноября','декабря'];
function fmtDate(iso){
  if(!iso)return{d:'',month:''};
  const d=new Date(iso+'T12:00:00');
  return{d:String(d.getDate()),month:MONTHS[d.getMonth()]};
}
function fmtTimeRange(time,end){
  if(!time)return'Без времени';
  return time+'–'+end;
}
function fmtMins(m){if(!m)return'~30м';if(m<60)return'~'+m+'м';const h=Math.floor(m/60),r=m%60;return r?(h+'ч '+r+'м'):(h+'ч');}
function fmtMinsExact(m){const h=Math.floor(m/60),r=m%60;return h+'ч '+(r<10?'0':'')+r+'м';}
function timeToMin(t){if(!t)return null;const[h,m]=t.split(':').map(Number);return h*60+m;}
function minToTime(m){const h=Math.floor(m/60),mm=m%60;return (h<10?'0':'')+h+':'+(mm<10?'0':'')+mm;}
function addMin(time,mins){const t=timeToMin(time);return minToTime(t+mins);}
function pad2(n){return n<10?'0'+n:''+n;}
function getWeekdayShort(iso){return WEEKDAYS_SHORT[new Date(iso+'T12:00:00').getDay()];}
function getWeekdayLong(iso){const n=new Date(iso+'T12:00:00').getDay();return['воскресенье','понедельник','вторник','среда','четверг','пятница','суббота'][n];}
function computeBlockDuration(b){if(b.time&&b.end)return timeToMin(b.end)-timeToMin(b.time);return b.duration||30;}

/* ── App shell — owns all shared state ── */
function App(){
  const[tab,setTab]=React.useState('today');

  /* Plan state (Plan v1) */
  const[unscheduled,setUnscheduled]=React.useState(INIT_UNSCHEDULED);
  const[scheduledMap,setScheduledMap]=React.useState(INIT_SCHEDULED);
  const[planUi,setPlanUi]=React.useState({mode:'week',selectedDate:TODAY});
  const[scheduleSheet,setScheduleSheet]=React.useState({open:false,mode:'schedule',task:null,block:null,sourceDate:null});
  const[taskSheet,setTaskSheet]=React.useState({open:false,block:null});

  /* Execution state — lifted to App so it survives tab switches.
     activeSession.kind:
       'planned'        — started from a scheduled block (taskId set)
       'unplanned'      — from "Делаю другое" pick (taskId null)
       'missed'         — legacy placeholder (no longer created in v1.1)
     Elapsed is derived: now - startedAt + accumulatedSec (while not paused). */
  const[activeSession,setActiveSession]=React.useState(null);
  const[factSessions,setFactSessions]=React.useState([]); /* completed/in-progress */
  const[dayCheckIn,setDayCheckIn]=React.useState(null);    /* {energy,focus,mood,obstacle} */
  const[dayMode,setDayMode]=React.useState('normal');      /* 'focus' | 'normal' | 'light' | 'recovery' */
  const[dayEnergy,setDayEnergy]=React.useState(0);         /* last reported energy 0..3 */
  const[dayStatus,setDayStatus]=React.useState('active');  /* current execution phase */
  const[todayTop,setTodayTop]=React.useState(['t3','t4','t5']);  /* taskId refs */
  const[quickNotesToday,setQuickNotesToday]=React.useState([]);
  const[endDayReview,setEndDayReview]=React.useState(null);

  /* For Сегодня to open Plan's ScheduleSheet as well */
  const openScheduleForBlock=(block,mode,sourceDate,options)=>{
    const src=sourceDate||TODAY;
    const opts=options||{};
    setScheduleSheet({open:true,mode:mode||'reschedule',task:null,block,sourceDate:src,
      initialDate:opts.initialDate||src,lockDate:!!opts.lockDate,purpose:opts.purpose||null});
  };

  /* Stable navigation callback for agent CTAs and cross-screen links. */
  const onNavigate=(to)=>{
    if(to==='plan')setTab('plan');
    else if(to==='today')setTab('today');
  };

  const appState={
    /* Plan */
    unscheduled,setUnscheduled,
    scheduledMap,setScheduledMap,
    planUi,setPlanUi,
    scheduleSheet,setScheduleSheet,
    taskSheet,setTaskSheet,
    /* Execution */
    activeSession,setActiveSession,
    factSessions,setFactSessions,
    dayCheckIn,setDayCheckIn,
    dayMode,setDayMode,
    dayEnergy,setDayEnergy,
    dayStatus,setDayStatus,
    todayTop,setTodayTop,
    quickNotesToday,setQuickNotesToday,
    endDayReview,setEndDayReview,
    /* helpers */
    onNavigate,
    openScheduleForBlock,
  };

  return(
    <div style={{minHeight:'100vh',background:'#04080D',padding:'24px 0',display:'flex',gap:24,
      fontFamily:T.font,alignItems:'flex-start',justifyContent:'center'}}>
      <DeviceFrame>
        <PhoneShell tab={tab} setTab={setTab}>
          {tab==='today'&&<TodayScreen appState={appState}/>}
          {tab==='plan'&&<PlanScreen appState={appState}/>}
          {tab==='projects'&&<StubScreen label="Проекты"/>}
          {tab==='time'&&<StubScreen label="Время"/>}
          {tab==='settings'&&<StubScreen label="Ещё"/>}
        </PhoneShell>
      </DeviceFrame>
      <DemoControls appState={appState} setTab={setTab}/>
    </div>
  );
}

/* ── Today screen — interactive execution dashboard ── */
function TodayScreen({appState}){
  const{unscheduled,setUnscheduled,scheduledMap,setScheduledMap,activeSession,setActiveSession,
    factSessions,setFactSessions,dayCheckIn,setDayCheckIn,dayMode,setDayMode,dayEnergy,setDayEnergy,
    dayStatus,setDayStatus,todayTop,setTodayTop,quickNotesToday,setQuickNotesToday,
    endDayReview,setEndDayReview,onNavigate,openScheduleForBlock,scheduleSheet,setScheduleSheet}=appState;

  /* Block lookup helpers by taskId (stable). */
  const todaysBlocks=scheduledMap[TODAY]||[];
  const blockByTaskId=(tid)=>todaysBlocks.find(b=>b.taskId===tid);
  const taskById=(tid)=>{
    const blk=blockByTaskId(tid);
    if(blk)return{taskId:blk.taskId,title:blk.title,project:blk.project,duration:computeBlockDuration(blk),color:blk.color};
    const u=unscheduled.find(t=>t.taskId===tid);
    if(u)return{taskId:u.taskId,title:u.title,project:u.project,duration:u.duration||30};
    return null;
  };

  /* Tick clock so active session elapsed updates. */
  const[now,setNow]=React.useState(Date.now());
  React.useEffect(()=>{
    if(!activeSession)return;
    const id=setInterval(()=>setNow(Date.now()),1000);
    return()=>clearInterval(id);
  },[activeSession]);

  /* Open sheets */
  const[openSheet,setOpenSheet]=React.useState(null);
  /* If "Делаю другое" first needs a schedule change, start the alternative
     activity only after ScheduleSheet confirms successfully. */
  const[pendingDoElse,setPendingDoElse]=React.useState(null);
  /* openSheet shapes:
     {kind:'checkin'} | {kind:'mode'} | {kind:'progress'}
     | {kind:'doElse',block}
     | {kind:'blockEnded',block}
     | {kind:'completion',block,energy}
     | {kind:'quickNote'}
     | {kind:'endDay'}
  */

  /* ── Execution state machine ── */
  /* Derived: if a session is active, the hero reflects it; otherwise the day
     status tells us what to show. */
  const heroState=(()=>{
    if(activeSession)return activeSession.paused?'paused':'active';
    if(dayStatus==='day_not_started')return'pre';
    if(dayStatus==='day_completed')return'done';
    if(dayStatus==='end_of_day')return'eod';
    const upcoming=todaysBlocks.find(b=>b.time&&timeToMin(b.time)>14*60+30&&b.progress<100);
    if(upcoming&&!activeSession&&dayStatus==='upcoming')return'upcoming';
    const ready=todaysBlocks.find(b=>b.time&&timeToMin(b.time)<=14*60+30&&timeToMin(b.end)>=14*60+30&&b.progress<100);
    if(ready&&dayStatus==='ready')return'ready';
    return'between';
  })();

  /* ── Handlers ── */
  function startBlock(block){
    setActiveSession({kind:'planned',taskId:block.taskId,blockId:block.id,startedAt:Date.now(),
      accumulatedSec:0,paused:false,time:block.time,end:block.end});
    setDayStatus('active');
  }
  function pauseToggle(){
    if(!activeSession)return;
    setActiveSession(s=>{
      const now=Date.now();
      if(s.paused){
        /* Resume: anchor startedAt at now; accumulatedMs already holds the time
           accrued before pause, so elapsed = (now - startedAt) + accumulatedMs. */
        return{...s,paused:false,startedAt:now,
          accumulatedSec:Math.floor((s.accumulatedMs||0)/1000)};
      }
      /* Pause: freeze accumulatedMs with the in-progress slice. */
      const sliceMs=now-s.startedAt;
      const totalMs=(s.accumulatedMs||0)+sliceMs;
      return{...s,paused:true,accumulatedMs:totalMs,
        accumulatedSec:Math.floor(totalMs/1000)};
    });
  }
  function blockEnd(){
    if(!activeSession||activeSession.kind!=='planned')return;
    const block=todaysBlocks.find(b=>b.id===activeSession.blockId);
    if(!block)return;
    setOpenSheet({kind:'blockEnded',block});
  }
  function resumeAfterPause(){
    if(!activeSession)return;
    /* Reuse pauseToggle so elapsed math stays consistent: it converts accumulatedMs
       into accumulatedSec and resets startedAt to Date.now(). */
    setActiveSession(s=>{
      const now=Date.now();
      return{...s,paused:false,startedAt:now,
        accumulatedSec:Math.floor((s.accumulatedMs||0)/1000)};
    });
  }
  function startDoElse(block){
    setOpenSheet({kind:'doElse',block});
  }
  function completeBlock(block,ease){
    /* Mark completed: progress=100. Closes active session if it was this one. */
    setScheduledMap(prev=>{
      const list=(prev[TODAY]||[]).map(b=>b.id===block.id?{...b,progress:100}:b);
      return{...prev,[TODAY]:list};
    });
    if(activeSession&&activeSession.blockId===block.id){
      setFactSessions(prev=>[...prev,{taskId:block.taskId,blockId:block.id,
        startedAt:activeSession.startedAt,endedAt:Date.now(),durationSec:elapsedSec,
        kind:'completed',ease:ease||'normal'}]);
      setActiveSession(null);
    }
  }
  function notDoneBlock(block){
    /* "Не делал" — does NOT record a factual session. Offers move/backlog. */
    if(activeSession&&activeSession.blockId===block.id){
      setActiveSession(null);
    }
    setOpenSheet({kind:'notDone',block});
  }
  function startAlternativeSession(originalBlock,selection){
    if(!selection)return;
    if(selection.kind==='other-task'&&selection.block){
      /* A real existing Today task keeps its stable taskId/blockId. */
      const b=selection.block;
      setActiveSession({kind:'planned',taskId:b.taskId,blockId:b.id,label:b.title,
        startedAt:Date.now(),accumulatedSec:0,accumulatedMs:0,paused:false,
        time:b.time,end:b.end,parentBlockId:originalBlock.id});
    }else{
      const fallback={
        unplanned:'Незапланированная работа',break:'Перерыв',personal:'Личное',other:'Другое'
      };
      setActiveSession({kind:'unplanned',taskId:null,blockId:null,
        label:selection.label||fallback[selection.kind]||'Другое',
        startedAt:Date.now(),accumulatedSec:0,accumulatedMs:0,paused:false,
        parentBlockId:originalBlock.id});
    }
    setDayStatus('active');
    setOpenSheet(null);
  }

  /* True two-step flow: first choose what is actually being done, then decide
     what to do with the original planned block. */
  function confirmDoElse(block,selection,handling){
    if(handling==='leave'){
      startAlternativeSession(block,selection);
      return;
    }
    setOpenSheet(null);
    setPendingDoElse({block,selection});
    if(handling==='shift'){
      openScheduleForBlock(block,'reschedule',TODAY,
        {initialDate:TODAY,lockDate:true,purpose:'doElse'});
    }else if(handling==='move'){
      openScheduleForBlock(block,'reschedule',TODAY,
        {initialDate:NEXT_DAY,lockDate:false,purpose:'doElse'});
    }
  }

  function finishActiveSession(){
    if(!activeSession)return;
    if(activeSession.kind==='planned'){
      blockEnd();
      return;
    }
    /* Unplanned/break/personal work is factual time only. It must never mark the
       original scheduled task completed. */
    setFactSessions(prev=>[...prev,{taskId:activeSession.taskId||null,
      blockId:activeSession.blockId||null,parentBlockId:activeSession.parentBlockId||null,
      label:activeSession.label||'Незапланированная работа',endedAt:Date.now(),
      durationSec:elapsedSec,kind:activeSession.kind||'unplanned'}]);
    setActiveSession(null);
    setDayStatus('between');
  }

  /* "Не делал" — record factual handled=missed only as a non-work marker
     so the user can keep the block visible in the timeline as not-done. */
  function notDoneHandling(block,action){
    if(action==='leave'){
      setOpenSheet(null);
    }else if(action==='move'){
      setOpenSheet(null);
      openScheduleForBlock(block,'reschedule',TODAY);
    }else if(action==='backlog'){
      /* Remove from today's blocks; keep taskId; insert as unscheduled (no date). */
      setScheduledMap(prev=>{
        const list=(prev[TODAY]||[]).filter(b=>b.id!==block.id);
        return{...prev,[TODAY]:list};
      });
      setUnscheduled(prev=>{
        if(prev.find(t=>t.taskId===block.taskId))return prev;
        return[...prev,{taskId:block.taskId,title:block.title,project:block.project,
          duration:computeBlockDuration(block),priority:'medium',plannedDate:null}];
      });
      setOpenSheet(null);
    }
  }

  /* End-of-day per-task handling: target is either 'tomorrow', a specific date, or 'backlog'.
     No fictitious 16:00 — only assign a time if the user explicitly picks a slot.
     Single-entity rule: a logical task lives either as a ScheduledBlock (has time)
     or as an unscheduled entry (date-only, no time). Never both for the same taskId
     in the same date — otherwise duplicates appear in Plan. */
  function endDayTaskHandle(block,action,opts){
    /* Helper: remove the block from TODAY and return its task meta. */
    const removeFromToday=(prev)=>{
      const list=(prev[TODAY]||[]).filter(b=>b.id!==block.id);
      return{...prev,[TODAY]:list};
    };
    /* Helper: ensure no unscheduled entry with this taskId exists for the same date. */
    const upsertUnscheduled=(prev,plannedDate)=>{
      const others=prev.filter(t=>t.taskId!==block.taskId);
      return[...others,{taskId:block.taskId,title:block.title,project:block.project,
        duration:computeBlockDuration(block),priority:'medium',plannedDate}];
    };
    if(action==='keep'){
      /* "Завтра" without a time = date-only unscheduled entry; nothing in scheduledMap. */
      setScheduledMap(removeFromToday);
      setUnscheduled(prev=>upsertUnscheduled(prev,NEXT_DAY));
    }else if(action==='backlog'){
      setScheduledMap(removeFromToday);
      setUnscheduled(prev=>upsertUnscheduled(prev,null));
    }else if(action==='date'){
      /* Move to a user-chosen date, no time = date-only unscheduled entry. */
      const targetDate=opts&&opts.date;
      if(!targetDate)return;
      setScheduledMap(removeFromToday);
      setUnscheduled(prev=>upsertUnscheduled(prev,targetDate));
    }else if(action==='time'){
      /* Keep the source placement intact until ScheduleSheet confirms. Cancel must be
         lossless. sourceDate remains TODAY; initialDate is only the proposed target. */
      const targetDate=opts&&opts.date?opts.date:NEXT_DAY;
      setOpenSheet(null);
      openScheduleForBlock(block,'reschedule',TODAY,
        {initialDate:targetDate,lockDate:false,purpose:'endDay'});
    }
  }

  function closeEndDay(){
    setOpenSheet(null);
  }

  function completeQuickNote(text,kind){
    if(!text.trim())return;
    setQuickNotesToday(prev=>[{text,kind:kind||'note',at:Date.now()},...prev].slice(0,5));
  }

  function checkInSubmit(payload){
    setDayCheckIn(payload);
    setDayEnergy(payload.energy||1);
    if(dayStatus==='day_not_started'||dayStatus==='upcoming')setDayStatus('active');
  }

  /* ── Render ── */
  const modeLabel={focus:'Фокус',normal:'Обычный',light:'Лёгкий',recovery:'Восстановление'}[dayMode];
  const totalToday=todaysBlocks.length;
  const doneToday=todaysBlocks.filter(b=>b.progress===100).length;
  const topTotal=todayTop.length;
  const topDone=todayTop.filter(tid=>{
    const b=blockByTaskId(tid);
    return b&&b.progress===100;
  }).length;
  const blockMin=todaysBlocks.reduce((s,b)=>s+computeBlockDuration(b),0);
  const doneMin=todaysBlocks.filter(b=>b.progress===100).reduce((s,b)=>s+computeBlockDuration(b),0);

  /* Agent insight — only when actionable. */
  const agent=(()=>{
    const overload=blockMin>8*60;
    const blockActive=blockByTaskId('t5');
    const lowEnergy=dayCheckIn&&dayCheckIn.energy===0&&blockActive&&!blockActive.progress;
    if(overload)return{title:'Перегруз дня',body:'Больше 8ч запланировано. Могу разнести задачи.',
      action:'Разгрузить',onClick:()=>onNavigate('plan')};
    if(lowEnergy)return{title:'Энергия низкая',body:'Сложный блок впереди — перенести или облегчить?',
      action:'Посмотреть',onClick:()=>onNavigate('plan')};
    return null;
  })();

  /* elapsed helper for active session.
     While running: (now - startedAt) + accumulatedMs.
     While paused :  accumulatedMs (frozen at pause). */
  const elapsedSec=activeSession?Math.max(0,
    Math.floor(((activeSession.paused?0:Date.now()-activeSession.startedAt)+(activeSession.accumulatedMs||0))/1000)):0;
  const fmtElapsed=(s)=>s<60?s+'с':(Math.floor(s/60)+'м');

  const currentBlock=activeSession&&activeSession.kind==='planned'
    ?todaysBlocks.find(b=>b.id===activeSession.blockId):null;
  const nextBlock=currentBlock
    ?null
    :todaysBlocks.find(b=>b.time&&timeToMin(b.time)>14*60+30&&b.progress<100);
  const upcomingBlock=nextBlock||todaysBlocks.find(b=>b.time&&b.progress<100);

  return(
    <div style={{height:'100%',overflowY:'auto',background:T.bg,position:'relative'}}>
      <TodayHeader dayCheckIn={dayCheckIn} onCheckIn={()=>setOpenSheet({kind:'checkin'})}
        dayStatus={dayStatus} onEndDay={()=>setOpenSheet({kind:'endDay'})}/>

      <div style={{padding:'14px 20px 100px'}}>
        <KpiRow modeLabel={modeLabel} dayMode={dayMode} onModeClick={()=>setOpenSheet({kind:'mode'})}
          doneToday={doneToday} totalToday={totalToday} topDone={topDone} topTotal={topTotal}
          doneMin={doneMin} blockMin={blockMin}
          energyLabel={dayCheckIn?(['⚡⚡⚡','⚡⚡','⚡','·'][dayCheckIn.energy||0]):'·'}
          onProgressClick={()=>setOpenSheet({kind:'progress'})}
          onEnergyClick={()=>setOpenSheet({kind:'checkin'})}/>

        <HeroBlock state={heroState} currentBlock={currentBlock} upcomingBlock={upcomingBlock}
          activeSession={activeSession} elapsedSec={elapsedSec} fmtElapsed={fmtElapsed}
          onStart={()=>{if(upcomingBlock)startBlock(upcomingBlock);}}
          onPauseToggle={pauseToggle} onEnd={finishActiveSession} onResume={resumeAfterPause}
          onDoElse={startDoElse} onNotDone={notDoneBlock}
          onCheckIn={()=>setOpenSheet({kind:'checkin'})}/>

        {agent&&(
          <AgentCard title={agent.title} body={agent.body} action={agent.action} onClick={agent.onClick}/>
        )}

        <TopThree todayTop={todayTop} blockByTaskId={blockByTaskId}
          activeSession={activeSession} startBlock={startBlock} todaysBlocks={todaysBlocks}
          onComplete={(b)=>{
            if(activeSession&&activeSession.blockId===b.id)setOpenSheet({kind:'completion',block:b});
            else completeBlock(b,'normal');
          }}/>

        <QuickNote quickNotesToday={quickNotesToday}
          onAdd={()=>setOpenSheet({kind:'quickNote'})}/>
      </div>

      {openSheet&&openSheet.kind==='checkin'&&(
        <CheckInSheet onSubmit={checkInSubmit} onClose={()=>setOpenSheet(null)} dayCheckIn={dayCheckIn}
          dayStatus={dayStatus} setDayStatus={setDayStatus}/>
      )}
      {openSheet&&openSheet.kind==='mode'&&(
        <ModeSheet onClose={()=>setOpenSheet(null)} dayMode={dayMode} setDayMode={setDayMode}
          setDayEnergy={setDayEnergy}/>
      )}
      {openSheet&&openSheet.kind==='progress'&&(
        <ProgressSheet onClose={()=>setOpenSheet(null)} todaysBlocks={todaysBlocks} todayTop={todayTop}
          doneMin={doneMin} blockMin={blockMin}/>
      )}
      {openSheet&&openSheet.kind==='doElse'&&(
        <DoSomethingElseSheet block={openSheet.block} todayBlocks={todaysBlocks}
          onClose={()=>setOpenSheet(null)}
          onConfirm={(selection,handling)=>confirmDoElse(openSheet.block,selection,handling)}/>
      )}
      {openSheet&&openSheet.kind==='blockEnded'&&(
        <BlockEndedSheet block={openSheet.block} onClose={()=>setOpenSheet(null)}
          onComplete={()=>{
            const b=openSheet.block;
            setOpenSheet({kind:'completion',block:b});
          }}
          onNotDone={()=>{
            const b=openSheet.block;
            setOpenSheet(null);
            notDoneBlock(b);
          }}
          onDoElse={()=>{
            /* "Ещё работаю": keep the same factual session running past plan end. */
            setOpenSheet(null);
          }}/>
      )}
      {openSheet&&openSheet.kind==='notDone'&&(
        <NotDoneSheet block={openSheet.block} onClose={()=>setOpenSheet(null)}
          onAction={(a)=>notDoneHandling(openSheet.block,a)}/>
      )}
      {openSheet&&openSheet.kind==='completion'&&(
        <CompletionSheet block={openSheet.block} onClose={()=>setOpenSheet(null)}
          onDone={(ease)=>{completeBlock(openSheet.block,ease);setOpenSheet(null);}}/>
      )}
      {openSheet&&openSheet.kind==='quickNote'&&(
        <QuickNoteSheet onClose={()=>setOpenSheet(null)} onSave={completeQuickNote}
          contextLabel={currentBlock?currentBlock.title:(activeSession?.label||null)}/>
      )}
      {openSheet&&openSheet.kind==='endDay'&&(
        <EndDayReviewSheet onClose={()=>setOpenSheet(null)}
          todaysBlocks={todaysBlocks} dayEnergy={dayEnergy} setDayEnergy={setDayEnergy}
          dayCheckIn={dayCheckIn} quickNotesToday={quickNotesToday} factSessions={factSessions}
          activeSession={activeSession} setActiveSession={setActiveSession}
          onTaskHandle={endDayTaskHandle} onFinish={()=>{
            if(activeSession){
              setFactSessions(prev=>[...prev,{taskId:activeSession.taskId||null,
                blockId:activeSession.blockId||null,parentBlockId:activeSession.parentBlockId||null,
                label:activeSession.label||null,endedAt:Date.now(),durationSec:elapsedSec,
                kind:'stopped_at_day_end'}]);
              setActiveSession(null);
            }
            setDayStatus('day_completed');setEndDayReview({at:Date.now(),energy:dayEnergy});
            setOpenSheet(null);
          }}/>
      )}

      {scheduleSheet.open&&<ScheduleSheet sheet={scheduleSheet} onConfirm={(p)=>{
        if(p.mode==='reschedule'&&p.block){
          const sourceDate=p.block.__sourceDate||scheduleSheet.sourceDate||TODAY;
          if(p.date===sourceDate){
            setScheduledMap(prev=>{
              const list=(prev[sourceDate]||[]).map(b=>b.id===p.block.id
                ?{...b,time:p.time,end:p.end}:b);
              return{...prev,[sourceDate]:list};
            });
          }else{
            setScheduledMap(prev=>{
              const fromList=(prev[sourceDate]||[]).filter(b=>b.id!==p.block.id);
              const toList=prev[p.date]||[];
              const movedBlock={...p.block,time:p.time,end:p.end};
              delete movedBlock.__sourceDate;
              return{...prev,[sourceDate]:fromList,[p.date]:[...toList,movedBlock]};
            });
          }
        }
        setScheduleSheet({open:false,mode:'schedule',task:null,block:null,sourceDate:null});
        if(pendingDoElse){
          const pending=pendingDoElse;
          setPendingDoElse(null);
          startAlternativeSession(pending.block,pending.selection);
        }
      }} onClose={()=>{
        setScheduleSheet({open:false,mode:'schedule',task:null,block:null,sourceDate:null});
        setPendingDoElse(null);
      }} scheduledMap={scheduledMap}/>}
    </div>
  );
}

/* ── Today sub-components ── */
function TodayHeader({dayCheckIn,onCheckIn,dayStatus,onEndDay}){
  return(
    <div style={{padding:'12px 20px 14px',background:'radial-gradient(120% 80% at 0% 0%, '+T.surface+' 0%, '+T.bg+' 70%)',
      borderBottom:'1px solid '+T.border}}>
      <div style={{display:'flex',alignItems:'baseline',justifyContent:'space-between'}}>
        <div>
          <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.06em',
            textTransform:'uppercase'}}>Сегодня</div>
          <div style={{fontSize:22,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:2}}>
            {TODAYS.d} · {TODAYS.dLong}
          </div>
        </div>
        <div style={{display:'flex',gap:8}}>
          {!dayCheckIn&&(
            <button onClick={onCheckIn} style={{padding:'8px 12px',borderRadius:10,
              background:T.accent+'22',border:'1px solid '+T.accent+'66',cursor:'pointer',
              fontSize:12,fontWeight:600,color:T.accent,fontFamily:T.font}}>Чек-ин</button>
          )}
          {dayCheckIn&&(
            <button onClick={onEndDay} style={{padding:'8px 12px',borderRadius:10,
              background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',
              fontSize:12,fontWeight:600,color:T.muted,fontFamily:T.font}}>Завершить день</button>
          )}
        </div>
      </div>
    </div>
  );
}

function KpiRow({modeLabel,dayMode,onModeClick,doneToday,totalToday,topDone,topTotal,
  doneMin,blockMin,energyLabel,onProgressClick,onEnergyClick}){
  return(
    <div style={{display:'flex',gap:8,marginBottom:14}}>
      <button onClick={onModeClick} style={{flex:1,textAlign:'left',
        background:T.surface,borderRadius:10,padding:'10px 12px',border:'1px solid '+T.border,cursor:'pointer'}}>
        <div style={{fontSize:10,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
          textTransform:'uppercase',marginBottom:2}}>Режим</div>
        <div style={{fontSize:14,fontWeight:600,color:T.fg,fontFamily:T.font}}>{modeLabel}</div>
      </button>
      <button onClick={onProgressClick} style={{flex:1,textAlign:'left',
        background:T.surface,borderRadius:10,padding:'10px 12px',border:'1px solid '+T.border,cursor:'pointer'}}>
        <div style={{fontSize:10,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
          textTransform:'uppercase',marginBottom:2}}>Прогресс</div>
        <div style={{fontSize:14,fontWeight:600,color:T.fg,fontFamily:T.font}}>
          {doneToday}/{totalToday} · {topDone}/{topTotal}
        </div>
      </button>
      <button onClick={onEnergyClick} style={{flex:1,textAlign:'left',
        background:T.surface,borderRadius:10,padding:'10px 12px',border:'1px solid '+T.border,cursor:'pointer'}}>
        <div style={{fontSize:10,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
          textTransform:'uppercase',marginBottom:2}}>Энергия</div>
        <div style={{fontSize:14,fontWeight:600,color:T.fg,fontFamily:T.font}}>{energyLabel}</div>
      </button>
    </div>
  );
}

function HeroBlock({state,currentBlock,upcomingBlock,activeSession,elapsedSec,fmtElapsed,
  onStart,onPauseToggle,onEnd,onResume,onDoElse,onNotDone,onCheckIn}){
  /* Pre-day */
  if(state==='pre'){
    return(
      <div style={{background:'linear-gradient(135deg,'+T.accentDeep+' 0%, '+T.bg+' 100%)',
        borderRadius:16,padding:'20px 18px',border:'1px solid '+T.accent+'55',marginBottom:16}}>
        <div style={{fontSize:11,color:T.accent,fontFamily:T.font,letterSpacing:'0.06em',
          textTransform:'uppercase',fontWeight:600}}>Скоро старт</div>
        <div style={{fontSize:18,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:6}}>
          Доброе утро
        </div>
        <div style={{fontSize:13,color:T.muted,fontFamily:T.font,marginTop:4,lineHeight:1.4}}>
          Перед началом — короткий чек-ин: энергия, фокус, препятствие.
        </div>
        <button onClick={onCheckIn} style={{marginTop:14,padding:'10px 16px',borderRadius:10,
          background:T.accent,border:'none',cursor:'pointer',fontSize:13,fontWeight:600,
          color:'#fff',fontFamily:T.font}}>Чек-ин</button>
      </div>
    );
  }
  /* Active block */
  if(state==='active'&&activeSession){
    return(
      <div style={{background:'linear-gradient(135deg,'+T.accentDeep+' 0%, '+T.bg+' 100%)',
        borderRadius:16,padding:'18px 18px',border:'1px solid '+T.accent,marginBottom:16,
        boxShadow:'0 0 0 1px '+T.accent+'22 inset'}}>
        <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',marginBottom:6}}>
          <span style={{fontSize:11,color:T.accent,fontFamily:T.font,letterSpacing:'0.06em',
            textTransform:'uppercase',fontWeight:600}}>Сейчас</span>
          <span style={{fontSize:11,color:T.muted,fontFamily:T.font,fontVariantNumeric:'tabular-nums'}}>
            {fmtElapsed(elapsedSec||0)}
          </span>
        </div>
        <div style={{fontSize:17,fontWeight:700,color:T.fg,fontFamily:T.font,lineHeight:1.2}}>
          {currentBlock?currentBlock.title:(activeSession.label||'Незапланированная работа')}
        </div>
        <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginTop:4}}>
          {currentBlock?`${currentBlock.time}–${currentBlock.end} · ${currentBlock.project}`:'Вне плана · фактическое время'}
        </div>
        <div style={{display:'flex',gap:8,marginTop:14,flexWrap:'wrap'}}>
          <button onClick={onPauseToggle} style={{padding:'8px 14px',borderRadius:10,
            background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',
            fontSize:12,fontWeight:600,color:T.fg,fontFamily:T.font}}>Пауза</button>
          <button onClick={onEnd} style={{padding:'8px 14px',borderRadius:10,
            background:T.accentGreen,border:'none',cursor:'pointer',
            fontSize:12,fontWeight:600,color:T.bg,fontFamily:T.font}}>Завершил</button>
          {currentBlock&&<button onClick={()=>onDoElse(currentBlock)} style={{padding:'8px 14px',borderRadius:10,
            background:'transparent',border:'1px solid '+T.border,cursor:'pointer',
            fontSize:12,fontWeight:500,color:T.muted,fontFamily:T.font}}>Делаю другое</button>}
        </div>
      </div>
    );
  }
  /* Paused */
  if(state==='paused'&&activeSession){
    return(
      <div style={{background:'linear-gradient(135deg,'+T.surface2+' 0%, '+T.bg+' 100%)',
        borderRadius:16,padding:'18px 18px',border:'1px solid '+T.border,marginBottom:16}}>
        <div style={{fontSize:11,color:T.accentOrange,fontFamily:T.font,letterSpacing:'0.06em',
          textTransform:'uppercase',fontWeight:600}}>На паузе</div>
        <div style={{fontSize:17,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:6,lineHeight:1.2}}>
          {currentBlock?currentBlock.title:(activeSession.label||'Незапланированная работа')}
        </div>
        <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginTop:4}}>
          Возобновите или завершите — время не идёт.
        </div>
        <div style={{display:'flex',gap:8,marginTop:14}}>
          <button onClick={onResume} style={{padding:'8px 14px',borderRadius:10,
            background:T.accent,border:'none',cursor:'pointer',
            fontSize:12,fontWeight:600,color:'#fff',fontFamily:T.font}}>Продолжить</button>
          <button onClick={onEnd} style={{padding:'8px 14px',borderRadius:10,
            background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',
            fontSize:12,fontWeight:500,color:T.muted,fontFamily:T.font}}>Завершить</button>
        </div>
      </div>
    );
  }
  /* Ready — current block window but no session */
  if(state==='ready'&&upcomingBlock){
    return(
      <div style={{background:'linear-gradient(135deg,'+T.accentDeep+' 0%, '+T.bg+' 100%)',
        borderRadius:16,padding:'18px 18px',border:'1px solid '+T.accent+'66',marginBottom:16}}>
        <div style={{fontSize:11,color:T.accent,fontFamily:T.font,letterSpacing:'0.06em',
          textTransform:'uppercase',fontWeight:600}}>Сейчас</div>
        <div style={{fontSize:17,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:6,lineHeight:1.2}}>
          {upcomingBlock.title}
        </div>
        <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginTop:4}}>
          {upcomingBlock.time}–{upcomingBlock.end} · {upcomingBlock.project}
        </div>
        <div style={{fontSize:12,color:T.accent,fontFamily:T.font,marginTop:10,fontWeight:600}}>
          Пора начинать
        </div>
        <div style={{display:'flex',gap:8,marginTop:14}}>
          <button onClick={onStart} style={{padding:'10px 18px',borderRadius:10,
            background:T.accent,border:'none',cursor:'pointer',
            fontSize:13,fontWeight:600,color:'#fff',fontFamily:T.font}}>Начать</button>
          <button onClick={()=>onDoElse(upcomingBlock)} style={{padding:'10px 14px',borderRadius:10,
            background:'transparent',border:'1px solid '+T.border,cursor:'pointer',
            fontSize:12,fontWeight:500,color:T.muted,fontFamily:T.font}}>Делаю другое</button>
        </div>
      </div>
    );
  }
  /* EOD */
  if(state==='eod'){
    return(
      <div style={{background:'linear-gradient(135deg,'+T.accentPurpleDeep+' 0%, '+T.bg+' 100%)',
        borderRadius:16,padding:'18px 18px',border:'1px solid '+T.accentPurple+'55',marginBottom:16}}>
        <div style={{fontSize:11,color:T.accentPurple,fontFamily:T.font,letterSpacing:'0.06em',
          textTransform:'uppercase',fontWeight:600}}>День почти закончен</div>
        <div style={{fontSize:17,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:6,lineHeight:1.2}}>
          Подведём итоги
        </div>
        <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginTop:4}}>
          Незакрытое перенесём в завтра или в бэклог.
        </div>
      </div>
    );
  }
  /* Done */
  if(state==='done'){
    return(
      <div style={{background:T.surface,borderRadius:16,padding:'18px 18px',
        border:'1px solid '+T.border,marginBottom:16}}>
        <div style={{fontSize:11,color:T.accentGreen,fontFamily:T.font,letterSpacing:'0.06em',
          textTransform:'uppercase',fontWeight:600}}>День завершён</div>
        <div style={{fontSize:17,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:6,lineHeight:1.2}}>
          Хороший день
        </div>
        <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginTop:4}}>
          Завтра — новый план.
        </div>
      </div>
    );
  }
  /* Between blocks */
  return(
    <div style={{background:'linear-gradient(135deg,'+T.surface2+' 0%, '+T.bg+' 100%)',
      borderRadius:16,padding:'18px 18px',border:'1px solid '+T.border,marginBottom:16}}>
      <div style={{fontSize:11,color:T.muted,fontFamily:T.font,letterSpacing:'0.06em',
        textTransform:'uppercase',fontWeight:600}}>Свободное окно</div>
      <div style={{fontSize:17,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:6,lineHeight:1.2}}>
        Перерыв
      </div>
      <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginTop:4}}>
        Можно отдохнуть или взять короткую задачу.
      </div>
    </div>
  );
}

function AgentCard({title,body,action,onClick}){
  return(
    <div style={{background:T.accentPurpleDeep,borderRadius:14,padding:14,
      border:'1px solid '+T.accentPurple+'55',display:'flex',alignItems:'center',gap:12,marginBottom:16}}>
      <div style={{width:36,height:36,borderRadius:18,background:T.accentPurple+'33',
        display:'flex',alignItems:'center',justifyContent:'center',flexShrink:0}}>
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke={T.accentPurple} strokeWidth="1.5">
          <path d="M9 1l2 4 4 .5-3 3 1 4-4-2-4 2 1-4-3-3 4-.5z"/>
        </svg>
      </div>
      <div style={{flex:1,minWidth:0}}>
        <div style={{fontSize:12,fontWeight:600,color:T.fg,fontFamily:T.font}}>{title}</div>
        <div style={{fontSize:11,color:T.muted,fontFamily:T.font,marginTop:2,lineHeight:1.35}}>{body}</div>
      </div>
      <button onClick={onClick} style={{padding:'6px 12px',borderRadius:8,
        background:T.accentPurple+'22',border:'1px solid '+T.accentPurple,
        color:T.accentPurple,fontSize:11,fontWeight:600,fontFamily:T.font,cursor:'pointer',
        flexShrink:0}}>{action}</button>
    </div>
  );
}

function TopThree({todayTop,blockByTaskId,activeSession,startBlock,todaysBlocks,onComplete}){
  const [expanded,setExpanded]=React.useState(false);
  const topItems=todayTop.map(tid=>blockByTaskId(tid)).filter(Boolean);
  /* "Показать все" toggles a Today-only full list (no nav to Plan, per memory). */
  const items=expanded
    ?todaysBlocks.filter(b=>b.progress<100)
    :topItems;
  return(
    <div style={{marginBottom:16}}>
      <div style={{display:'flex',alignItems:'baseline',justifyContent:'space-between',marginBottom:8}}>
        <div style={{fontSize:12,fontWeight:600,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
          textTransform:'uppercase'}}>Главное на сегодня</div>
        <button onClick={()=>setExpanded(e=>!e)} style={{background:'none',border:'none',cursor:'pointer',
          fontSize:11,color:T.accent,fontFamily:T.font,fontWeight:500}}>
          {expanded?'Скрыть':'Показать все'}
        </button>
      </div>
      {items.map(b=>{
        const isActive=activeSession&&activeSession.blockId===b.id;
        return(
          <div key={b.taskId} style={{display:'flex',alignItems:'center',padding:'10px 12px',
            background:isActive?T.accent+'14':T.surface,borderRadius:10,marginBottom:6,
            border:'1px solid '+(isActive?T.accent+'55':T.border)}}>
            <button onClick={()=>{if(b.progress<100&&onComplete)onComplete(b);}}
              style={{width:22,height:22,borderRadius:11,flexShrink:0,marginRight:10,
              background:b.progress===100?T.accentGreen:T.surface2,
              border:'1.5px solid '+(b.progress===100?T.accentGreen:T.border),
              display:'flex',alignItems:'center',justifyContent:'center',cursor:'pointer',padding:0}}>
              {b.progress===100&&(
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke={T.bg} strokeWidth="2">
                  <polyline points="2,6 5,9 10,3"/></svg>
              )}
            </button>
            <div style={{flex:1,minWidth:0}}>
              <div style={{fontSize:14,fontWeight:500,color:T.fg,fontFamily:T.font,
                textDecoration:b.progress===100?'line-through':'none',
                opacity:b.progress===100?0.7:1,overflow:'hidden',textOverflow:'ellipsis',
                whiteSpace:'nowrap'}}>{b.title}</div>
              <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>
                {b.time}–{b.end}
              </div>
            </div>
            {!activeSession&&!isActive&&b.progress<100&&(
              <button onClick={()=>startBlock(b)} style={{padding:'6px 10px',borderRadius:8,
                background:T.accent+'22',border:'1px solid '+T.accent+'55',cursor:'pointer',
                fontSize:11,fontWeight:600,color:T.accent,fontFamily:T.font,flexShrink:0}}>Начать</button>
            )}
            {isActive&&(
              <span style={{fontSize:11,fontWeight:600,color:T.accent,fontFamily:T.font}}>идёт</span>
            )}
          </div>
        );
      })}
    </div>
  );
}

function QuickNote({quickNotesToday,onAdd}){
  return(
    <div>
      <div style={{display:'flex',alignItems:'baseline',justifyContent:'space-between',marginBottom:8}}>
        <div style={{fontSize:12,fontWeight:600,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
          textTransform:'uppercase'}}>Быстрая заметка</div>
        <button onClick={onAdd} style={{background:'none',border:'none',cursor:'pointer',
          fontSize:11,color:T.accent,fontFamily:T.font,fontWeight:500}}>+ Добавить</button>
      </div>
      {quickNotesToday.length===0?(
        <div style={{padding:'12px 14px',background:T.surface,borderRadius:10,
          border:'1px dashed '+T.border,color:T.muted2,fontSize:12,fontFamily:T.font,textAlign:'center'}}>
          Здесь появятся заметки дня
        </div>
      ):(
        <div style={{display:'flex',flexDirection:'column',gap:6}}>
          {quickNotesToday.slice(0,3).map((n,i)=>(
            <div key={i} style={{padding:'8px 12px',background:T.surface,borderRadius:10,
              border:'1px solid '+T.border,fontSize:12,color:T.muted,fontFamily:T.font}}>
              {n.text}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/* ── Today sheets ── */
function Sheet({onClose,title,children}){
  return(
    <div onClick={onClose} style={{position:'absolute',inset:0,background:'rgba(0,0,0,0.55)',
      display:'flex',alignItems:'flex-end',zIndex:50}}>
      <div onClick={e=>e.stopPropagation()} style={{width:'100%',maxHeight:'85%',
        background:T.surface,borderTopLeftRadius:18,borderTopRightRadius:18,
        border:'1px solid '+T.border,overflowY:'auto',paddingBottom:24}}>
        <div style={{display:'flex',justifyContent:'center',padding:'8px 0'}}>
          <div style={{width:36,height:4,borderRadius:2,background:T.border}}/>
        </div>
        {title&&(
          <div style={{padding:'6px 20px 12px',fontSize:16,fontWeight:600,color:T.fg,fontFamily:T.font}}>
            {title}
          </div>
        )}
        {children}
      </div>
    </div>
  );
}

function CheckInSheet({onSubmit,onClose,dayCheckIn,dayStatus,setDayStatus}){
  const[energy,setEnergy]=React.useState(dayCheckIn?dayCheckIn.energy:2);
  const[focus,setFocus]=React.useState(dayCheckIn?dayCheckIn.focus:2);
  const[mood,setMood]=React.useState(dayCheckIn?dayCheckIn.mood:'ok');
  const[obstacle,setObstacle]=React.useState(dayCheckIn?dayCheckIn.obstacle:'');
  return(
    <Sheet onClose={onClose} title="Чек-ин">
      <div style={{padding:'0 20px'}}>
        <Label>Энергия</Label>
        <EmojiScale value={energy} onChange={setEnergy} labels={['·','⚡','⚡⚡','⚡⚡⚡']}/>
        <Label style={{marginTop:18}}>Фокус</Label>
        <EmojiScale value={focus} onChange={setFocus} labels={['туман','средний','острый','пик']}/>
        <Label style={{marginTop:18}}>Самочувствие</Label>
        <MoodPicker value={mood} onChange={setMood}/>
        <Label style={{marginTop:18}}>Что может помешать?</Label>
        <textarea value={obstacle} onChange={e=>setObstacle(e.target.value)} rows={2} placeholder="Необязательно"
          style={{width:'100%',background:T.surface2,border:'1px solid '+T.border,borderRadius:10,
            padding:10,color:T.fg,fontFamily:T.font,fontSize:13,resize:'none',boxSizing:'border-box'}}/>
        <div style={{display:'flex',gap:8,marginTop:20}}>
          <button onClick={()=>{
            onSubmit({energy,focus,mood,obstacle});
            if(dayStatus==='day_not_started')setDayStatus('active');
            onClose();
          }} style={{flex:1,padding:'12px 0',borderRadius:10,background:T.accent,border:'none',
            cursor:'pointer',color:'#fff',fontSize:14,fontWeight:600,fontFamily:T.font}}>
            {dayStatus==='day_not_started'?'Начать день':'Сохранить'}
          </button>
        </div>
      </div>
    </Sheet>
  );
}

function Label({children,style}){
  return <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
    textTransform:'uppercase',fontWeight:600,marginBottom:8,...(style||{})}}>{children}</div>;
}

function EmojiScale({value,onChange,labels}){
  return(
    <div style={{display:'flex',gap:6}}>
      {labels.map((l,i)=>(
        <button key={i} onClick={()=>onChange(i)} style={{flex:1,padding:'10px 0',borderRadius:10,
          background:i===value?T.accent+'22':T.surface2,
          border:'1px solid '+(i===value?T.accent:T.border),cursor:'pointer',
          color:i===value?T.fg:T.muted,fontSize:13,fontWeight:600,fontFamily:T.font}}>
          {l}
        </button>
      ))}
    </div>
  );
}

function MoodPicker({value,onChange}){
  const opts=[{id:'bad',label:'Плохо',color:T.accentRed},
    {id:'ok',label:'Нормально',color:T.muted},
    {id:'good',label:'Хорошо',color:T.accentGreen}];
  return(
    <div style={{display:'flex',gap:6}}>
      {opts.map(o=>(
        <button key={o.id} onClick={()=>onChange(o.id)} style={{flex:1,padding:'10px 0',borderRadius:10,
          background:value===o.id?o.color+'22':T.surface2,
          border:'1px solid '+(value===o.id?o.color:T.border),cursor:'pointer',
          color:value===o.id?T.fg:T.muted,fontSize:13,fontWeight:600,fontFamily:T.font}}>
          {o.label}
        </button>
      ))}
    </div>
  );
}

function ModeSheet({onClose,dayMode,setDayMode,setDayEnergy}){
  const opts=[
    {id:'focus',label:'Фокус',desc:'Глубокая работа, минимум прерываний',color:T.accentPurple},
    {id:'normal',label:'Обычный',desc:'Сбалансированный день',color:T.accent},
    {id:'light',label:'Лёгкий',desc:'Только ключевые задачи',color:T.accentGreen},
    {id:'recovery',label:'Восстановление',desc:'Отдых и короткие задачи',color:T.accentOrange},
  ];
  return(
    <Sheet onClose={onClose} title="Режим дня">
      <div style={{padding:'0 20px',display:'flex',flexDirection:'column',gap:8}}>
        {opts.map(o=>(
          <button key={o.id} onClick={()=>{setDayMode(o.id);onClose();}} style={{textAlign:'left',
            padding:'12px 14px',borderRadius:12,background:dayMode===o.id?o.color+'22':T.surface2,
            border:'1px solid '+(dayMode===o.id?o.color:T.border),cursor:'pointer'}}>
            <div style={{fontSize:14,fontWeight:600,color:T.fg,fontFamily:T.font}}>{o.label}</div>
            <div style={{fontSize:11,color:T.muted,fontFamily:T.font,marginTop:2}}>{o.desc}</div>
          </button>
        ))}
      </div>
    </Sheet>
  );
}

function ProgressSheet({onClose,todaysBlocks,todayTop,doneMin,blockMin}){
  const done=todaysBlocks.filter(b=>b.progress===100).length;
  const total=todaysBlocks.length;
  const topDone=todayTop.filter(tid=>{
    const b=todaysBlocks.find(x=>x.taskId===tid);
    return b&&b.progress===100;
  }).length;
  return(
    <Sheet onClose={onClose} title="Прогресс">
      <div style={{padding:'0 20px'}}>
        <div style={{background:T.surface2,borderRadius:12,padding:14,border:'1px solid '+T.border,
          marginBottom:10}}>
          <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
            textTransform:'uppercase',marginBottom:4}}>Задачи</div>
          <div style={{fontSize:22,fontWeight:700,color:T.fg,fontFamily:T.font}}>{done}/{total}</div>
        </div>
        <div style={{background:T.surface2,borderRadius:12,padding:14,border:'1px solid '+T.border,
          marginBottom:10}}>
          <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
            textTransform:'uppercase',marginBottom:4}}>Главное</div>
          <div style={{fontSize:22,fontWeight:700,color:T.fg,fontFamily:T.font}}>{topDone}/{todayTop.length}</div>
        </div>
        <div style={{background:T.surface2,borderRadius:12,padding:14,border:'1px solid '+T.border}}>
          <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
            textTransform:'uppercase',marginBottom:4}}>Время</div>
          <div style={{fontSize:22,fontWeight:700,color:T.fg,fontFamily:T.font}}>
            {fmtMinsExact(doneMin)} / {fmtMinsExact(blockMin)}
          </div>
        </div>
      </div>
    </Sheet>
  );
}

function DoSomethingElseSheet({block,todayBlocks,onClose,onConfirm}){
  const[stage,setStage]=React.useState('pick'); /* pick | task | handling */
  const[selected,setSelected]=React.useState(null);
  const[customLabel,setCustomLabel]=React.useState('');
  const opts=[
    {id:'other-task',label:'Другая задача',desc:'Из списка на сегодня'},
    {id:'unplanned',label:'Незапланированная',desc:'Вне плана'},
    {id:'break',label:'Перерыв',desc:'Короткий отдых'},
    {id:'personal',label:'Личное',desc:'Личная задача'},
    {id:'other',label:'Другое',desc:'Свой вариант'},
  ];
  const handlingOpts=[
    {id:'leave',label:'Оставить в плане',desc:'Ничего не менять'},
    {id:'shift',label:'Сдвинуть',desc:'Изменить время сегодня'},
    {id:'move',label:'Перенести',desc:'На другой день или время'},
  ];
  const alternativeTasks=(todayBlocks||[]).filter(b=>b.id!==block.id&&b.progress<100&&b.time);
  function chooseKind(o){
    if(o.id==='other-task'){
      setStage('task');
      return;
    }
    setSelected({kind:o.id,label:o.label});
    setStage('handling');
  }
  function chooseTask(b){
    setSelected({kind:'other-task',label:b.title,block:b});
    setStage('handling');
  }
  function confirmHandling(h){
    if(!selected)return;
    const label=customLabel.trim()||selected.label;
    onConfirm({...selected,label},h);
  }
  return(
    <Sheet onClose={onClose} title="Делаю другое">
      <div style={{padding:'0 20px'}}>
        <div style={{fontSize:13,color:T.muted,fontFamily:T.font,marginBottom:12,lineHeight:1.4}}>
          Текущий блок: <span style={{color:T.fg,fontWeight:600}}>{block.title}</span> · {block.time}–{block.end}
        </div>
        {stage==='pick'&&(<>
          <Label>Что делаете вместо?</Label>
          <div style={{display:'flex',flexDirection:'column',gap:6}}>
            {opts.map(o=>(
              <button key={o.id} onClick={()=>chooseKind(o)} style={{textAlign:'left',
                padding:'10px 12px',borderRadius:10,background:T.surface2,border:'1px solid '+T.border,
                cursor:'pointer'}}>
                <div style={{fontSize:13,fontWeight:600,color:T.fg,fontFamily:T.font}}>{o.label}</div>
                <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>{o.desc}</div>
              </button>
            ))}
          </div>
        </>)}
        {stage==='task'&&(<>
          <Label>Выберите задачу</Label>
          <div style={{display:'flex',flexDirection:'column',gap:6,marginBottom:12}}>
            {alternativeTasks.length===0&&(
              <div style={{padding:'10px 12px',background:T.surface2,borderRadius:10,
                color:T.muted2,fontSize:12,fontFamily:T.font}}>Других незавершённых задач сегодня нет.</div>
            )}
            {alternativeTasks.map(b=>(
              <button key={b.id} onClick={()=>chooseTask(b)} style={{textAlign:'left',padding:'10px 12px',
                borderRadius:10,background:T.surface2,border:'1px solid '+T.border,cursor:'pointer'}}>
                <div style={{fontSize:13,fontWeight:600,color:T.fg,fontFamily:T.font}}>{b.title}</div>
                <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>
                  {b.time}–{b.end} · {b.project}
                </div>
              </button>
            ))}
          </div>
          <button onClick={()=>setStage('pick')} style={{background:'none',border:'none',color:T.accent,
            fontSize:12,fontFamily:T.font,cursor:'pointer'}}>← Назад</button>
        </>)}
        {stage==='handling'&&selected&&(<>
          <div style={{padding:'10px 12px',background:T.accent+'12',border:'1px solid '+T.accent+'35',
            borderRadius:10,marginBottom:14}}>
            <div style={{fontSize:11,color:T.muted2,fontFamily:T.font}}>Фактически сейчас</div>
            <div style={{fontSize:13,fontWeight:600,color:T.fg,fontFamily:T.font,marginTop:2}}>{selected.label}</div>
          </div>
          {(selected.kind==='unplanned'||selected.kind==='other'||selected.kind==='personal')&&(
            <input value={customLabel} onChange={e=>setCustomLabel(e.target.value)}
              placeholder="Уточнить название (необязательно)" style={{width:'100%',boxSizing:'border-box',
                padding:'10px 12px',background:T.surface2,border:'1px solid '+T.border,borderRadius:10,
                color:T.fg,fontSize:13,fontFamily:T.font,marginBottom:14}}/>
          )}
          <Label>Что сделать с исходным блоком?</Label>
          <div style={{display:'flex',flexDirection:'column',gap:6}}>
            {handlingOpts.map(o=>(
              <button key={o.id} onClick={()=>confirmHandling(o.id)} style={{textAlign:'left',
                padding:'10px 12px',borderRadius:10,background:T.surface2,border:'1px solid '+T.border,
                cursor:'pointer'}}>
                <div style={{fontSize:13,fontWeight:600,color:T.fg,fontFamily:T.font}}>{o.label}</div>
                <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>{o.desc}</div>
              </button>
            ))}
          </div>
          <button onClick={()=>{setSelected(null);setCustomLabel('');setStage('pick');}}
            style={{marginTop:12,background:'none',border:'none',color:T.accent,fontSize:12,
              fontFamily:T.font,cursor:'pointer'}}>← Изменить действие</button>
        </>)}
      </div>
    </Sheet>
  );
}

function BlockEndedSheet({block,onClose,onComplete,onNotDone,onDoElse}){
  return(
    <Sheet onClose={onClose} title="Блок закончился">
      <div style={{padding:'0 20px'}}>
        <div style={{fontSize:14,color:T.fg,fontFamily:T.font,marginBottom:14,fontWeight:600}}>
          {block.title}
        </div>
        <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginBottom:18}}>
          {block.time}–{block.end} · {block.project}
        </div>
        <div style={{display:'flex',flexDirection:'column',gap:8}}>
          <button onClick={onComplete} style={{padding:'12px 0',borderRadius:10,
            background:T.accentGreen,border:'none',cursor:'pointer',
            color:T.bg,fontSize:14,fontWeight:600,fontFamily:T.font}}>Завершил</button>
          <button onClick={onDoElse} style={{padding:'12px 0',borderRadius:10,
            background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',
            color:T.fg,fontSize:13,fontWeight:600,fontFamily:T.font}}>Ещё работаю</button>
          <button onClick={onNotDone} style={{padding:'12px 0',borderRadius:10,
            background:'transparent',border:'1px solid '+T.border,cursor:'pointer',
            color:T.muted,fontSize:13,fontWeight:500,fontFamily:T.font}}>Не делал</button>
        </div>
      </div>
    </Sheet>
  );
}

function NotDoneSheet({block,onClose,onAction}){
  const opts=[
    {id:'move',label:'Перенести',desc:'На завтра или другой день',color:T.accent},
    {id:'backlog',label:'В бэклог',desc:'Без даты, вернуть позже',color:T.accentOrange},
    {id:'leave',label:'Оставить',desc:'Оставить блок как есть',color:T.muted},
  ];
  return(
    <Sheet onClose={onClose} title="Не делал">
      <div style={{padding:'0 20px'}}>
        <div style={{fontSize:13,color:T.muted,fontFamily:T.font,marginBottom:14,lineHeight:1.4}}>
          {block.title} · {block.time}–{block.end}
        </div>
        <Label>Что делаем с блоком?</Label>
        <div style={{display:'flex',flexDirection:'column',gap:8}}>
          {opts.map(o=>(
            <button key={o.id} onClick={()=>onAction(o.id)} style={{textAlign:'left',
              padding:'12px 14px',borderRadius:12,background:o.color+'18',border:'1px solid '+o.color+'55',
              cursor:'pointer'}}>
              <div style={{fontSize:14,fontWeight:600,color:T.fg,fontFamily:T.font}}>{o.label}</div>
              <div style={{fontSize:11,color:T.muted,fontFamily:T.font,marginTop:2}}>{o.desc}</div>
            </button>
          ))}
        </div>
      </div>
    </Sheet>
  );
}

function CompletionSheet({block,onClose,onDone}){
  const[ease,setEase]=React.useState('normal');
  const opts=[
    {id:'easy',label:'Легко',color:T.accentGreen},
    {id:'normal',label:'Нормально',color:T.accent},
    {id:'hard',label:'Тяжело',color:T.accentOrange},
  ];
  return(
    <Sheet onClose={onClose} title="Готово">
      <div style={{padding:'0 20px'}}>
        <div style={{fontSize:14,color:T.fg,fontFamily:T.font,marginBottom:6,fontWeight:600}}>
          {block.title}
        </div>
        <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginBottom:18}}>
          План · {block.time}–{block.end} · {block.project}
        </div>
        <Label>Как ощущалось?</Label>
        <div style={{display:'flex',gap:6,marginBottom:18}}>
          {opts.map(o=>(
            <button key={o.id} onClick={()=>setEase(o.id)} style={{flex:1,padding:'10px 0',borderRadius:10,
              background:ease===o.id?o.color+'22':T.surface2,
              border:'1px solid '+(ease===o.id?o.color:T.border),cursor:'pointer',
              color:ease===o.id?T.fg:T.muted,fontSize:13,fontWeight:600,fontFamily:T.font}}>
              {o.label}
            </button>
          ))}
        </div>
        <button onClick={()=>onDone(ease)} style={{width:'100%',padding:'12px 0',borderRadius:10,
          background:T.accent,border:'none',cursor:'pointer',color:'#fff',fontSize:14,fontWeight:600,
          fontFamily:T.font}}>Готово</button>
      </div>
    </Sheet>
  );
}

function QuickNoteSheet({onClose,onSave,contextLabel}){
  const[text,setText]=React.useState('');
  const[kind,setKind]=React.useState('note');
  const kinds=[
    {id:'note',label:'Заметка'},
    {id:'idea',label:'Идея'},
    {id:'problem',label:'Проблема'},
    {id:'summary',label:'Итог'},
  ];
  return(
    <Sheet onClose={onClose} title="Быстрая заметка">
      <div style={{padding:'0 20px'}}>
        {contextLabel&&(
          <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
            textTransform:'uppercase',marginBottom:8}}>
            Контекст: {contextLabel}
          </div>
        )}
        <Label>Тип</Label>
        <div style={{display:'flex',gap:6,marginBottom:14,flexWrap:'wrap'}}>
          {kinds.map(k=>(
            <button key={k.id} onClick={()=>setKind(k.id)} style={{padding:'6px 12px',borderRadius:8,
              background:kind===k.id?T.accent+'22':T.surface2,
              border:'1px solid '+(kind===k.id?T.accent:T.border),cursor:'pointer',
              color:kind===k.id?T.fg:T.muted,fontSize:12,fontWeight:600,fontFamily:T.font}}>
              {k.label}
            </button>
          ))}
        </div>
        <Label>Текст</Label>
        <textarea value={text} onChange={e=>setText(e.target.value)} rows={4} autoFocus
          placeholder="Что важно зафиксировать?"
          style={{width:'100%',background:T.surface2,border:'1px solid '+T.border,borderRadius:10,
            padding:10,color:T.fg,fontFamily:T.font,fontSize:13,resize:'none',boxSizing:'border-box'}}/>
        <button onClick={()=>{onSave(text,kind);setText('');onClose();}} disabled={!text.trim()}
          style={{width:'100%',marginTop:14,padding:'12px 0',borderRadius:10,
            background:text.trim()?T.accent:T.surface2,border:'none',cursor:text.trim()?'pointer':'default',
            color:text.trim()?'#fff':T.muted,fontSize:14,fontWeight:600,fontFamily:T.font}}>
          Сохранить
        </button>
      </div>
    </Sheet>
  );
}

function EndDayReviewSheet({onClose,todaysBlocks,dayEnergy,setDayEnergy,dayCheckIn,
  quickNotesToday,factSessions,activeSession,setActiveSession,onTaskHandle,onFinish}){
  const unfinished=todaysBlocks.filter(b=>b.progress<100);
  return(
    <Sheet onClose={onClose} title="Итоги дня">
      <div style={{padding:'0 20px'}}>
        <div style={{display:'flex',gap:8,marginBottom:18}}>
          <KpiMini label="Сделано" value={todaysBlocks.filter(b=>b.progress===100).length+'/'+todaysBlocks.length} color={T.accentGreen}/>
          <KpiMini label="Заметок" value={String(quickNotesToday.length)} color={T.accent}/>
          <KpiMini label="Факт-сессий" value={String(factSessions.length+(activeSession?1:0))} color={T.accentPurple}/>
        </div>

        {unfinished.length>0&&(
          <>
            <Label>Незакрытое</Label>
            {unfinished.map(b=>(
              <UnfinishedRow key={b.id} block={b} onHandle={(a,opts)=>onTaskHandle(b,a,opts)}/>
            ))}
          </>
        )}

        <Label style={{marginTop:18}}>Энергия в конце</Label>
        <EmojiScale value={dayEnergy} onChange={setDayEnergy} labels={['·','⚡','⚡⚡','⚡⚡⚡']}/>

        <button onClick={onFinish} style={{width:'100%',marginTop:20,padding:'12px 0',borderRadius:10,
          background:T.accent,border:'none',cursor:'pointer',color:'#fff',fontSize:14,fontWeight:600,
          fontFamily:T.font}}>Завершить день</button>
      </div>
    </Sheet>
  );
}

function UnfinishedRow({block,onHandle}){
  const[open,setOpen]=React.useState(false);
  return(
    <div style={{background:T.surface,borderRadius:10,border:'1px solid '+T.border,marginBottom:6,
      overflow:'hidden'}}>
      <div style={{padding:'10px 12px',display:'flex',alignItems:'center'}}>
        <div style={{flex:1,minWidth:0}}>
          <div style={{fontSize:13,fontWeight:500,color:T.fg,fontFamily:T.font,
            overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap'}}>{block.title}</div>
          <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>
            {block.time}–{block.end}
          </div>
        </div>
        <button onClick={()=>setOpen(!open)} style={{padding:'6px 10px',borderRadius:8,
          background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',
          fontSize:11,color:T.muted,fontFamily:T.font,flexShrink:0}}>
          {open?'Скрыть':'Действие'}
        </button>
      </div>
      {open&&(
        <div style={{padding:'6px 12px 12px',display:'flex',flexDirection:'column',gap:6,
          background:T.bg}}>
          <RowAction label="Завтра" onClick={()=>onHandle('keep')}/>
          <RowAction label="В бэклог" onClick={()=>onHandle('backlog')}/>
          <RowAction label="На дату" onClick={()=>onHandle('date',{date:TODAYS.addDays(2)})}/>
          <RowAction label="Выбрать дату и время" onClick={()=>onHandle('time',{date:NEXT_DAY})}/>
        </div>
      )}
    </div>
  );
}

function RowAction({label,onClick}){
  return(
    <button onClick={onClick} style={{textAlign:'left',padding:'8px 10px',borderRadius:8,
      background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',
      fontSize:12,color:T.fg,fontFamily:T.font,fontWeight:500}}>
      {label}
    </button>
  );
}

/* ── Plan v1 — preserved exactly as previously validated ── */
function PlanScreen({appState}){
  const{unscheduled,setUnscheduled,scheduledMap,setScheduledMap,planUi,setPlanUi,
    scheduleSheet,setScheduleSheet,taskSheet,setTaskSheet}=appState;
  const planMode=planUi.mode;
  const setPlanMode=(m)=>setPlanUi(p=>({...p,mode:m}));
  const selectedDate=planUi.selectedDate;
  const setSelectedDate=(d)=>setPlanUi(p=>({...p,selectedDate:d}));

  const dayBlocks=scheduledMap[selectedDate]||[];
  const dayUnscheduled=unscheduled.filter(t=>t.plannedDate===selectedDate&&!t.postponed);
  const backlogNoDate=unscheduled.filter(t=>!t.plannedDate&&!t.postponed);
  const backlogHasDate=unscheduled.filter(t=>t.plannedDate&&!t.postponed);
  const backlogPostponed=unscheduled.filter(t=>t.postponed);

  const[viewMonth,setViewMonth]=React.useState(new Date(TODAY+'T12:00:00'));

  function openScheduleForTask(task){
    setScheduleSheet({open:true,mode:'schedule',task,block:null,sourceDate:selectedDate});
  }
  function openScheduleForBlock(block,mode,sourceDate,options){
    const sd=sourceDate||selectedDate;
    const opts=options||{};
    setScheduleSheet({open:true,mode:mode||'reschedule',task:null,block:{...block,__sourceDate:sd},
      sourceDate:sd,initialDate:opts.initialDate||sd,lockDate:!!opts.lockDate,purpose:opts.purpose||null});
  }
  function openTaskForBlock(block){
    setTaskSheet({open:true,block:{...block,__sourceDate:selectedDate}});
  }
  function confirmSchedule({mode,task,block,date,time,end}){
    if(mode==='schedule'&&task){
      const newId='s_'+task.taskId+'_'+date.replaceAll('-','');
      const newBlock={id:newId,taskId:task.taskId,title:task.title,project:task.project,
        time:time||null,end:end||null,color:pickColor(task.priority,task.project),progress:0};
      setScheduledMap(prev=>({...prev,[date]:[...(prev[date]||[]),newBlock]}));
      /* Once a task has an actual time placement it must not also remain in the
         date-only/unscheduled pool, regardless of target date. */
      setUnscheduled(prev=>prev.filter(t=>t.taskId!==task.taskId));
    }else if(mode==='reschedule'&&block){
      if(date===block.__sourceDate){
        setScheduledMap(prev=>{
          const list=(prev[date]||[]).map(b=>b.id===block.id?{...b,time:time||b.time,end:end||b.end}:b);
          return{...prev,[date]:list};
        });
      }else{
        setScheduledMap(prev=>{
          const fromList=(prev[block.__sourceDate]||[]).filter(b=>b.id!==block.id);
          const toList=prev[date]||[];
          const movedBlock={...block,time:time||null,end:end||null};
          return{...prev,[block.__sourceDate]:fromList,[date]:[...toList,movedBlock]};
        });
      }
    }
    setScheduleSheet({open:false,mode:'schedule',task:null,block:null,sourceDate:null});
  }
  function closeSchedule(){
    setScheduleSheet({open:false,mode:'schedule',task:null,block:null,sourceDate:null});
  }
  function pickColor(priority,project){
    if(project==='Обучение')return T.accentPurple;
    if(project==='Self Development')return T.accentPurple;
    if(priority==='high')return T.accent;
    if(priority==='low')return T.accentOrange;
    return T.accent;
  }
  function completeBlock(blockId,date){
    setScheduledMap(prev=>{
      const list=(prev[date]||[]).map(b=>b.id===blockId?{...b,progress:100}:b);
      return{...prev,[date]:list};
    });
  }
  function agentMoveTask({task,direction}){
    /* Date-only moves stay in the unscheduled pool. A ScheduledBlock is created
       only after a real time slot is chosen. */
    setUnscheduled(prev=>{
      const others=prev.filter(t=>t.taskId!==task.taskId);
      if(direction==='backlog')return[...others,{...task,plannedDate:null,postponed:false}];
      if(direction==='tomorrow')return[...others,{...task,plannedDate:NEXT_DAY,postponed:false}];
      return prev;
    });
  }
  function postpone(task){
    setUnscheduled(prev=>prev.map(t=>t.taskId===task.taskId?{...t,postponed:true,plannedDate:null}:t));
  }

  return(
    <div style={{height:'100%',overflowY:'auto',background:T.bg,position:'relative'}}>
      <PlanHeader viewMonth={viewMonth} setViewMonth={setViewMonth} planMode={planMode} setPlanMode={setPlanMode}/>
      {planMode==='day'&&(
        <PlanDayView dayBlocks={dayBlocks} dayUnscheduled={dayUnscheduled}
          selectedDate={selectedDate} setSelectedDate={setSelectedDate} scheduledMap={scheduledMap}
          openTaskForBlock={openTaskForBlock} openScheduleForBlock={openScheduleForBlock}
          openScheduleForTask={openScheduleForTask} completeBlock={completeBlock} date={selectedDate}
          setScheduledMap={setScheduledMap}/>
      )}
      {planMode==='week'&&(
        <PlanWeekView viewMonth={viewMonth} selectedDate={selectedDate} setSelectedDate={setSelectedDate}
          scheduledMap={scheduledMap} unscheduled={unscheduled}
          openTaskForBlock={openTaskForBlock} openScheduleForBlock={openScheduleForBlock}
          completeBlock={completeBlock} setPlanMode={setPlanMode} date={selectedDate}/>
      )}
      {planMode==='backlog'&&(
        <PlanBacklogView backlogNoDate={backlogNoDate} backlogHasDate={backlogHasDate}
          backlogPostponed={backlogPostponed} openScheduleForTask={openScheduleForTask}
          setUnscheduled={setUnscheduled} TODAY={TODAY}/>
      )}
      <PlanAgent scheduledMap={scheduledMap} setScheduledMap={setScheduledMap} setUnscheduled={setUnscheduled} date={TODAY}
        onAction={(action)=>{if(action==='Открыть'||action==='Разгрузить день')setPlanMode('day');}}/>
      {scheduleSheet.open&&<ScheduleSheet sheet={scheduleSheet} onConfirm={confirmSchedule} onClose={closeSchedule} scheduledMap={scheduledMap}/>}
      {taskSheet.open&&<TaskDetailsSheet sheet={taskSheet} onClose={()=>setTaskSheet({open:false,block:null})}
        onReschedule={(b,mode,date,options)=>{setTaskSheet({open:false,block:null});
          openScheduleForBlock(b,mode,date,options);}}
        onComplete={(b)=>{completeBlock(b.id,b.__sourceDate||selectedDate);setTaskSheet({open:false,block:null});}}/>}
    </div>
  );
}

function PlanHeader({viewMonth,setViewMonth,planMode,setPlanMode}){
  const monthName=MONTHS_LONG[viewMonth.getMonth()];
  const year=viewMonth.getFullYear();
  function shiftMonth(n){const d=new Date(viewMonth);d.setMonth(d.getMonth()+n);setViewMonth(d);}
  return(
    <div style={{padding:'16px 20px 12px',background:'linear-gradient(180deg,'+T.surface+' 0%, '+T.bg+' 100%)',
      borderBottom:'1px solid '+T.border}}>
      <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',marginBottom:12}}>
        <div>
          <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
            textTransform:'uppercase'}}>План</div>
          <div style={{fontSize:22,fontWeight:700,color:T.fg,fontFamily:T.font,marginTop:2}}>
            {monthName} {year}
          </div>
        </div>
        <div style={{display:'flex',gap:6}}>
          <button onClick={()=>shiftMonth(-1)} style={headerBtn()}><Chevron/></button>
          <button onClick={()=>shiftMonth(1)} style={{...headerBtn(),transform:'rotate(180deg)'}}><Chevron/></button>
        </div>
      </div>
      <SegmentedControl value={planMode} onChange={setPlanMode}
        options={[{id:'day',label:'День'},{id:'week',label:'Неделя'},{id:'backlog',label:'Бэклог'}]}/>
    </div>
  );
}
function headerBtn(){
  return{width:34,height:34,borderRadius:10,background:T.surface2,border:'1px solid '+T.border,
    color:T.muted,cursor:'pointer',display:'flex',alignItems:'center',justifyContent:'center'};
}
function Chevron(){
  return <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke={T.muted} strokeWidth="2">
    <polyline points="9,3 5,7 9,11"/></svg>;
}
function SegmentedControl({value,onChange,options}){
  return(
    <div style={{display:'flex',background:T.surface2,borderRadius:10,padding:3,border:'1px solid '+T.border}}>
      {options.map(o=>(
        <button key={o.id} onClick={()=>onChange(o.id)} style={{flex:1,padding:'8px 0',borderRadius:8,border:'none',
          background:value===o.id?T.surface3:'transparent',color:value===o.id?T.fg:T.muted2,
          fontSize:13,fontWeight:value===o.id?600:500,fontFamily:T.font,cursor:'pointer',
          transition:'all 0.15s'}}>
          {o.label}
        </button>
      ))}
    </div>
  );
}

function PlanDayView({dayBlocks,dayUnscheduled,selectedDate,setSelectedDate,scheduledMap,
  openTaskForBlock,openScheduleForBlock,openScheduleForTask,completeBlock,date,setScheduledMap}){
  return(
    <div style={{padding:'12px 20px 80px'}}>
      <DayPickerStrip selectedDate={selectedDate} setSelectedDate={setSelectedDate}/>
      <div style={{marginTop:12,marginBottom:14,display:'flex',alignItems:'baseline',gap:8}}>
        <div style={{fontSize:13,color:T.muted2,fontFamily:T.font}}>
          {getWeekdayLong(selectedDate)}
        </div>
        <div style={{fontSize:18,fontWeight:600,color:T.fg,fontFamily:T.font}}>
          {fmtDate(selectedDate).d} {MONTHS_LONG[new Date(selectedDate+'T12:00:00').getMonth()]}
        </div>
      </div>
      <DayWorkloadSummary blocks={dayBlocks} unscheduled={dayUnscheduled}/>
      <DayTimeline blocks={dayBlocks} openTaskForBlock={openTaskForBlock}
        openScheduleForBlock={openScheduleForBlock} completeBlock={completeBlock} date={date}
        setScheduledMap={setScheduledMap}/>
      {dayUnscheduled.length>0&&(
        <div style={{marginTop:20}}>
          <div style={{fontSize:12,fontWeight:600,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
            textTransform:'uppercase',marginBottom:8}}>Без времени · {dayUnscheduled.length}</div>
          {dayUnscheduled.map(t=>(
            <div key={t.taskId} style={{display:'flex',alignItems:'center',padding:'10px 12px',
              background:T.surface,borderRadius:10,marginBottom:6,border:'1px solid '+T.border}}>
              <div style={{flex:1,minWidth:0}}>
                <div style={{fontSize:14,fontWeight:500,color:T.fg,fontFamily:T.font,
                  overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap'}}>{t.title}</div>
                <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>
                  {t.project} · {fmtMins(t.duration)}
                </div>
              </div>
              <button onClick={()=>openScheduleForTask(t)} style={{width:34,height:34,borderRadius:8,
                background:T.accent+'18',border:'1px solid '+T.accent+'40',display:'flex',
                alignItems:'center',justifyContent:'center',cursor:'pointer',flexShrink:0,marginLeft:8}}>
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke={T.accent} strokeWidth="1.5">
                  <rect x="2" y="3" width="12" height="11" rx="2"/>
                  <line x1="5" y1="1" x2="5" y2="5"/><line x1="11" y1="1" x2="11" y2="5"/>
                  <line x1="2" y1="7" x2="14" y2="7"/>
                  <line x1="8" y1="9" x2="8" y2="13"/><line x1="6" y1="11" x2="10" y2="11"/>
                </svg>
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
function DayPickerStrip({selectedDate,setSelectedDate}){
  const days=[];
  const today=new Date(TODAY+'T12:00:00');
  for(let i=-3;i<=10;i++){const d=new Date(today);d.setDate(d.getDate()+i);days.push(d);}
  return(
    <div style={{display:'flex',gap:6,overflowX:'auto',paddingBottom:4,scrollbarWidth:'none'}}>
      {days.map((d,i)=>{
        const iso=d.toISOString().slice(0,10);
        const isSel=iso===selectedDate;
        const isT=iso===TODAY;
        return(
          <button key={i} onClick={()=>setSelectedDate(iso)} style={{flexShrink:0,display:'flex',flexDirection:'column',
            alignItems:'center',padding:'8px 12px',minWidth:46,borderRadius:10,
            background:isSel?T.accent+'22':T.surface,border:isSel?'1px solid '+T.accent+'66':'1px solid '+T.border,
            cursor:'pointer'}}>
            <span style={{fontSize:10,color:isSel?T.accent:T.muted2,fontFamily:T.font,fontWeight:600,
              textTransform:'uppercase',letterSpacing:'0.04em'}}>{WEEKDAYS_SHORT[d.getDay()]}</span>
            <span style={{fontSize:16,fontWeight:600,color:isSel?T.fg:T.muted,fontFamily:T.font,marginTop:2}}>
              {d.getDate()}
            </span>
            {isT&&!isSel&&<div style={{width:4,height:4,borderRadius:2,background:T.accent,marginTop:2}}/>}
          </button>
        );
      })}
    </div>
  );
}
function DayWorkloadSummary({blocks,unscheduled}){
  const blockMin=blocks.reduce((s,b)=>s+computeBlockDuration(b),0);
  const uMin=unscheduled.reduce((s,t)=>s+(t.duration||0),0);
  const total=blockMin+uMin;
  const overload=total>8*60;
  return(
    <div style={{display:'flex',gap:8,marginBottom:14}}>
      <KpiMini label="Запланировано" value={fmtMinsExact(blockMin)} color={T.accent}/>
      <KpiMini label="Без времени" value={fmtMinsExact(uMin)} color={T.accentOrange}/>
      <KpiMini label={overload?'Перегруз':'Нагрузка'} value={fmtMinsExact(total)} color={overload?T.accentRed:T.accentGreen}/>
    </div>
  );
}
function KpiMini({label,value,color}){
  return(
    <div style={{flex:1,background:T.surface,borderRadius:10,padding:'8px 10px',border:'1px solid '+T.border}}>
      <div style={{fontSize:10,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
        textTransform:'uppercase',marginBottom:2}}>{label}</div>
      <div style={{fontSize:14,fontWeight:600,color:color||T.fg,fontFamily:T.font}}>{value}</div>
    </div>
  );
}
function DayTimeline({blocks,openTaskForBlock,openScheduleForBlock,completeBlock,date,setScheduledMap}){
  const sorted=[...blocks].sort((a,b)=>timeToMin(a.time||'99:99')-timeToMin(b.time||'99:99'));
  return(
    <div style={{position:'relative',paddingLeft:50}}>
      <div style={{position:'absolute',left:18,top:6,bottom:6,width:1,background:T.border}}/>
      {sorted.map(b=>{
        const startMin=timeToMin(b.time||'09:00');
        const dur=computeBlockDuration(b);
        const top=(startMin-(9*60))*(40/60);
        const height=Math.max(28,dur*(40/60));
        const done=b.progress===100;
        return(
          <div key={b.id} onClick={()=>openTaskForBlock(b)} style={{position:'relative',paddingBottom:6,cursor:'pointer'}}>
            <div style={{position:'absolute',left:-38,top:6,fontSize:11,color:T.muted2,fontFamily:T.font,
              width:30,textAlign:'right',fontVariantNumeric:'tabular-nums'}}>{b.time||'·'}</div>
            <div style={{background:done?T.surface:T.surface2,borderRadius:10,padding:'8px 12px',height:height,
              border:done?'1px solid '+T.border:'1px solid '+T.accent+'33',display:'flex',alignItems:'center',
              opacity:done?0.55:1}}>
              <div style={{width:3,height:Math.max(16,height-16),borderRadius:2,
                background:done?T.muted2:b.color||T.accent,marginRight:10,flexShrink:0}}/>
              <div style={{flex:1,minWidth:0}}>
                <div style={{fontSize:13,fontWeight:done?500:600,color:done?T.muted:T.fg,fontFamily:T.font,
                  textDecoration:done?'line-through':'none',overflow:'hidden',textOverflow:'ellipsis',
                  whiteSpace:'nowrap'}}>{b.title}</div>
                <div style={{fontSize:10,color:T.muted2,fontFamily:T.font,marginTop:2}}>
                  {b.project}{b.time?' · '+b.time+'–'+b.end:''}
                </div>
                {!done&&b.progress>0&&(
                  <div style={{height:2,background:T.border,borderRadius:1,marginTop:4,overflow:'hidden'}}>
                    <div style={{height:'100%',width:b.progress+'%',background:T.accent,borderRadius:1}}/>
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
      {sorted.length===0&&(
        <div style={{padding:'24px 0',textAlign:'center',color:T.muted2,fontSize:13,fontFamily:T.font}}>
          Пустой день — добавьте задачу
        </div>
      )}
    </div>
  );
}

function PlanWeekView({viewMonth,selectedDate,setSelectedDate,scheduledMap,unscheduled,
  openTaskForBlock,openScheduleForBlock,completeBlock,setPlanMode,date}){
  const monthStart=new Date(viewMonth.getFullYear(),viewMonth.getMonth(),1);
  const monthEnd=new Date(viewMonth.getFullYear(),viewMonth.getMonth()+1,0);
  const firstDayOfWeek=(monthStart.getDay()+6)%7;
  const cells=[];
  for(let i=0;i<firstDayOfWeek;i++)cells.push(null);
  for(let d=1;d<=monthEnd.getDate();d++)cells.push(new Date(viewMonth.getFullYear(),viewMonth.getMonth(),d));
  while(cells.length%7!==0)cells.push(null);
  return(
    <div style={{padding:'12px 16px 80px'}}>
      <DayPickerStrip selectedDate={selectedDate} setSelectedDate={setSelectedDate}/>
      <div style={{marginTop:14,fontSize:13,color:T.muted2,fontFamily:T.font,textAlign:'center'}}>
        Неделя · кликните день для детального плана
      </div>
      <div style={{marginTop:12,display:'grid',gridTemplateColumns:'repeat(7,1fr)',gap:4,marginBottom:8}}>
        {['Пн','Вт','Ср','Чт','Пт','Сб','Вс'].map(d=>(
          <div key={d} style={{textAlign:'center',fontSize:10,color:T.muted2,fontFamily:T.font,
            textTransform:'uppercase',letterSpacing:'0.04em',padding:'4px 0'}}>{d}</div>
        ))}
      </div>
      <div style={{display:'grid',gridTemplateColumns:'repeat(7,1fr)',gap:4}}>
        {cells.map((d,i)=>{
          if(!d)return <div key={i} style={{aspectRatio:'1/1'}}/>;
          const iso=d.toISOString().slice(0,10);
          const blocks=scheduledMap[iso]||[];
          const hasToday=iso===TODAY;
          const isSel=iso===selectedDate;
          const isPastDate=iso<TODAY;
          return(
            <button key={i} onClick={()=>{setSelectedDate(iso);setPlanMode('day');}} style={{aspectRatio:'1/1',
              background:isSel?T.accent+'22':hasToday?T.accent+'10':T.surface,borderRadius:8,
              border:isSel?'1px solid '+T.accent:hasToday?'1px solid '+T.accent+'40':'1px solid '+T.border,
              display:'flex',flexDirection:'column',alignItems:'center',padding:'4px 2px',cursor:'pointer',
              opacity:isPastDate?0.55:1}}>
              <span style={{fontSize:13,fontWeight:hasToday?700:500,color:hasToday?T.accent:T.fg,
                fontFamily:T.font}}>{d.getDate()}</span>
              {blocks.length>0&&(
                <div style={{display:'flex',gap:2,marginTop:2}}>
                  {blocks.slice(0,3).map((b,j)=>(
                    <div key={j} style={{width:4,height:4,borderRadius:1,background:b.color||T.accent}}/>
                  ))}
                </div>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}

function PlanBacklogView({backlogNoDate,backlogHasDate,backlogPostponed,openScheduleForTask,setUnscheduled,TODAY}){
  return(
    <div style={{padding:'12px 20px 80px'}}>
      <div style={{marginBottom:18}}>
        <div style={{fontSize:12,fontWeight:600,color:T.muted2,fontFamily:T.font,marginBottom:8,
          textTransform:'uppercase',letterSpacing:'0.04em'}}>Без даты · {backlogNoDate.length}</div>
        {backlogNoDate.map(t=>(
          <div key={t.taskId} style={{display:'flex',alignItems:'center',padding:'12px 14px',
            background:T.surface,borderRadius:10,marginBottom:6,border:'1px solid '+T.border}}>
            <div style={{flex:1,minWidth:0}}>
              <div style={{fontSize:14,fontWeight:500,color:T.fg,fontFamily:T.font}}>{t.title}</div>
              <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>
                {t.project} · {fmtMins(t.duration)}
              </div>
            </div>
            <button onClick={()=>openScheduleForTask(t)} style={{padding:'6px 12px',borderRadius:8,
              background:T.accent+'18',border:'1px solid '+T.accent+'40',cursor:'pointer',
              fontSize:12,color:T.accent,fontWeight:500,fontFamily:T.font,flexShrink:0,marginLeft:8}}>
              В план
            </button>
          </div>
        ))}
      </div>
      <div style={{marginBottom:18}}>
        <div style={{fontSize:12,fontWeight:600,color:T.muted2,fontFamily:T.font,marginBottom:8,
          textTransform:'uppercase',letterSpacing:'0.04em'}}>Есть дата · {backlogHasDate.length}</div>
        {backlogHasDate.map(t=>{
          const tf=fmtDate(t.plannedDate);
          return(
            <div key={t.taskId} style={{display:'flex',alignItems:'center',padding:'12px 14px',
              background:T.surface,borderRadius:10,marginBottom:6,border:'1px solid '+T.border}}>
              <div style={{flex:1,minWidth:0}}>
                <div style={{fontSize:14,fontWeight:500,color:T.fg,fontFamily:T.font}}>{t.title}</div>
                <div style={{display:'flex',gap:8,marginTop:4}}>
                  <span style={{fontSize:11,color:T.muted2,fontFamily:T.font}}>{t.project}</span>
                  <span style={{fontSize:11,color:T.muted2,fontFamily:T.font}}>~{fmtMins(t.duration)}</span>
                  <span style={{fontSize:11,color:T.accent,fontFamily:T.font}}>
                    {tf.d} {tf.month}
                  </span>
                </div>
              </div>
              <button onClick={()=>openScheduleForTask(t)} style={{padding:'6px 12px',borderRadius:8,
                background:T.accent+'18',border:'1px solid '+T.accent+'40',cursor:'pointer',
                fontSize:12,color:T.accent,fontWeight:500,fontFamily:T.font,flexShrink:0,marginLeft:8}}>
                В план
              </button>
            </div>
          );
        })}
      </div>
      <div style={{marginBottom:16}}>
        <div style={{fontSize:12,fontWeight:600,color:T.muted2,fontFamily:T.font,marginBottom:8,
          textTransform:'uppercase',letterSpacing:'0.04em'}}>Отложенные · {backlogPostponed.length}</div>
        {backlogPostponed.map(t=>(
          <div key={t.taskId} style={{display:'flex',alignItems:'center',padding:'12px 14px',
            background:T.surface,borderRadius:10,marginBottom:6,border:'1px solid '+T.border,opacity:0.7}}>
            <div style={{flex:1,minWidth:0}}>
              <div style={{fontSize:14,fontWeight:500,color:T.fg,fontFamily:T.font,textDecoration:'line-through'}}>{t.title}</div>
              <div style={{display:'flex',gap:8,marginTop:4}}>
                <span style={{fontSize:11,color:T.muted2,fontFamily:T.font}}>{t.project}</span>
                <span style={{fontSize:11,color:T.muted2,fontFamily:T.font}}>~{fmtMins(t.duration)}</span>
                <span style={{fontSize:11,color:T.accentOrange,fontFamily:T.font,fontWeight:500}}>Отложено</span>
              </div>
            </div>
            <button onClick={()=>{
              setUnscheduled(prev=>prev.map(task=>task.taskId===t.taskId?{...task,postponed:false,plannedDate:TODAY}:task));
            }} style={{padding:'6px 12px',borderRadius:8,
              background:T.accentOrange+'18',border:'1px solid '+T.accentOrange+'40',cursor:'pointer',
              fontSize:12,color:T.accentOrange,fontWeight:500,fontFamily:T.font,flexShrink:0,marginLeft:8}}>
              Вернуть
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

function PlanAgent({scheduledMap,setScheduledMap,setUnscheduled,date,onAction}){
  const blocks=scheduledMap[date]||[];
  const totalMin=blocks.reduce((s,b)=>s+computeBlockDuration(b),0);
  const overload=totalMin>8*60;
  const unfinished=blocks.filter(b=>b.progress<100&&timeToMin(b.end||'99:99')<14*60+30);
  const insight=overload
    ?{title:'Перегруз дня',body:'Больше 8ч запланировано. Могу разнести задачи.',
      action:'Разгрузить',color:T.accentOrange}
    :unfinished.length>0
      ?{title:'Не закрыт блок',body:unfinished[0].title+' — оцените, успеваете?',
        action:'Открыть',color:T.accentOrange}
      :null;
  if(!insight)return null;
  return(
    <div style={{position:'absolute',bottom:80,left:16,right:16,background:T.accentPurpleDeep,
      borderRadius:14,padding:14,border:'1px solid '+T.accentPurple+'55',display:'flex',alignItems:'center',gap:12}}>
      <div style={{width:36,height:36,borderRadius:18,background:T.accentPurple+'33',
        display:'flex',alignItems:'center',justifyContent:'center',flexShrink:0}}>
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke={T.accentPurple} strokeWidth="1.5">
          <path d="M9 1l2 4 4 .5-3 3 1 4-4-2-4 2 1-4-3-3 4-.5z"/>
        </svg>
      </div>
      <div style={{flex:1,minWidth:0}}>
        <div style={{fontSize:12,fontWeight:600,color:T.fg,fontFamily:T.font}}>{insight.title}</div>
        <div style={{fontSize:11,color:T.muted,fontFamily:T.font,marginTop:2}}>{insight.body}</div>
      </div>
      <button onClick={()=>onAction&&onAction(insight.action)} style={{padding:'6px 12px',borderRadius:8,
        background:T.accentPurple+'22',border:'1px solid '+T.accentPurple,
        color:T.accentPurple,fontSize:11,fontWeight:600,fontFamily:T.font,cursor:'pointer',flexShrink:0}}>
        {insight.action}
      </button>
    </div>
  );
}

function ScheduleSheet({sheet,onConfirm,onClose,scheduledMap}){
  const{mode,task,block,sourceDate}=sheet;
  const initialDate=sheet.initialDate||(mode==='reschedule'?sourceDate:(task?.plannedDate||TODAY));
  const lockDate=!!sheet.lockDate;
  const[date,setDate]=React.useState(initialDate);
  const[time,setTime]=React.useState(block?.time||null);
  const[end,setEnd]=React.useState(block?.end||null);
  const target=mode==='reschedule'?block:task;
  const existingBlocks=(scheduledMap[date]||[]).filter(b=>b.id!==block?.id);
  const dur=mode==='reschedule'?computeBlockDuration(block):(task?.duration||30);
  const dateChoices=[];
  [sourceDate,TODAY,NEXT_DAY,TODAYS.addDays(2)].filter(Boolean).forEach(d=>{
    if(!dateChoices.includes(d))dateChoices.push(d);
  });
  function dateLabel(d){
    if(d===TODAY)return'Сегодня';
    if(d===NEXT_DAY)return'Завтра';
    const f=fmtDate(d);return getWeekdayShort(d)+' · '+f.d+' '+f.month;
  }
  function chooseDate(d){
    if(lockDate)return;
    setDate(d);setTime(null);setEnd(null);
  }
  function suggest(timeMin){
    const e=timeMin+dur;
    return{time:minToTime(timeMin),end:minToTime(e)};
  }
  function applySuggestion(t,e){setTime(t);setEnd(e);}
  function adjustTime(deltaMin){
    if(!time){setTime('09:00');setEnd(addMin('09:00',dur));return;}
    const newT=addMin(time,deltaMin);setTime(newT);setEnd(addMin(newT,dur));
  }
  function setManualTime(t){setTime(t);if(t)setEnd(addMin(t,dur));}
  function manualEnd(e){if(time&&e&&timeToMin(e)>timeToMin(time))setEnd(e);}
  const slots=[];
  for(let m=9*60;m<19*60;m+=30){
    const e=m+dur;if(e>19*60)break;
    const conflict=existingBlocks.some(b=>{
      if(!b.time||!b.end)return false;
      const bs=timeToMin(b.time),be=timeToMin(b.end);return m<be&&e>bs;
    });
    if(!conflict)slots.push(suggest(m));
  }
  const tStart=timeToMin(time),tEnd=timeToMin(end);
  const inRange=tStart!==null&&tEnd!==null&&tStart>=9*60&&tEnd<=19*60&&tEnd>tStart;
  const overlap=inRange&&existingBlocks.some(b=>{
    if(!b.time||!b.end)return false;
    const bs=timeToMin(b.time),be=timeToMin(b.end);return tStart<be&&tEnd>bs;
  });
  const canConfirm=!!date&&inRange&&!overlap;
  function confirm(){
    if(!canConfirm)return;
    if(mode==='reschedule')onConfirm({mode:'reschedule',block:{...block,__sourceDate:sourceDate},date,time,end});
    else onConfirm({mode:'schedule',task,date,time,end});
  }
  return(
    <Sheet onClose={onClose} title={mode==='reschedule'?'Изменить расписание':'Запланировать'}>
      <div style={{padding:'0 20px 20px'}}>
        {target&&(
          <div style={{padding:'12px 14px',background:T.surface,borderRadius:10,border:'1px solid '+T.border,
            marginBottom:14}}>
            <div style={{fontSize:14,fontWeight:600,color:T.fg,fontFamily:T.font}}>{target.title||target.name}</div>
            <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,marginTop:2}}>
              {target.project} · {fmtMins(dur)}
            </div>
          </div>
        )}
        <Label>Дата</Label>
        <div style={{display:'flex',gap:6,flexWrap:'wrap',marginBottom:14}}>
          {(lockDate?[date]:dateChoices).map(d=>(
            <button key={d} onClick={()=>chooseDate(d)} style={{padding:'7px 10px',borderRadius:8,
              background:date===d?T.accent+'22':T.surface2,border:'1px solid '+(date===d?T.accent:T.border),
              color:date===d?T.fg:T.muted,fontSize:12,fontWeight:600,fontFamily:T.font,
              cursor:lockDate?'default':'pointer'}}>{dateLabel(d)}</button>
          ))}
        </div>
        <Label>Свободные слоты</Label>
        <div style={{display:'flex',flexWrap:'wrap',gap:6,marginBottom:14}}>
          {slots.length===0&&(<div style={{fontSize:12,color:T.muted,fontFamily:T.font}}>Нет свободных слотов</div>)}
          {slots.slice(0,8).map((sl,i)=>(
            <button key={i} onClick={()=>applySuggestion(sl.time,sl.end)} style={{padding:'6px 10px',borderRadius:8,
              background:time===sl.time&&end===sl.end?T.accent+'30':T.accent+'18',
              border:'1px solid '+T.accent+'40',cursor:'pointer',color:T.accent,fontSize:12,
              fontWeight:500,fontFamily:T.font}}>{sl.time}–{sl.end}</button>
          ))}
        </div>
        <Label>Время вручную</Label>
        <div style={{display:'flex',alignItems:'center',gap:6,marginBottom:10}}>
          <button onClick={()=>adjustTime(-60)} style={{width:34,height:34,borderRadius:8,
            background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',color:T.muted,fontSize:12,
            fontFamily:T.font}}>−1ч</button>
          <button onClick={()=>adjustTime(-15)} style={{width:34,height:34,borderRadius:8,
            background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',color:T.muted,fontSize:12,
            fontFamily:T.font}}>−15</button>
          <input value={time||''} onChange={e=>setManualTime(e.target.value)} placeholder="09:00"
            style={{flex:1,background:T.surface2,border:'1px solid '+T.border,borderRadius:8,padding:8,
              color:T.fg,fontFamily:T.font,fontSize:13,textAlign:'center',minWidth:0}}/>
          <span style={{color:T.muted2,fontFamily:T.font,fontSize:13}}>–</span>
          <input value={end||''} onChange={e=>manualEnd(e.target.value)} placeholder="10:00"
            style={{flex:1,background:T.surface2,border:'1px solid '+T.border,borderRadius:8,padding:8,
              color:T.fg,fontFamily:T.font,fontSize:13,textAlign:'center',minWidth:0}}/>
          <button onClick={()=>adjustTime(15)} style={{width:34,height:34,borderRadius:8,
            background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',color:T.muted,fontSize:12,
            fontFamily:T.font}}>+15</button>
          <button onClick={()=>adjustTime(60)} style={{width:34,height:34,borderRadius:8,
            background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',color:T.muted,fontSize:12,
            fontFamily:T.font}}>+1ч</button>
        </div>
        {!canConfirm&&time&&end&&(
          <div style={{fontSize:11,color:T.accentRed,fontFamily:T.font,marginBottom:10}}>
            {!inRange?'Время должно быть в пределах 09:00–19:00.':'Этот интервал пересекается с другим блоком.'}
          </div>
        )}
        <button disabled={!canConfirm} onClick={confirm} style={{width:'100%',padding:'12px 0',borderRadius:10,
          background:canConfirm?T.accent:T.surface2,border:'none',cursor:canConfirm?'pointer':'default',
          color:canConfirm?'#fff':T.muted2,fontSize:14,fontWeight:600,fontFamily:T.font}}>
          {mode==='reschedule'?(date===sourceDate?'Сохранить':'Перенести'):'Запланировать'}
        </button>
      </div>
    </Sheet>
  );
}

function TaskDetailsSheet({sheet,onClose,onReschedule,onComplete}){
  const{block}=sheet;
  if(!block)return null;
  return(
    <Sheet onClose={onClose} title="Задача">
      <div style={{padding:'0 20px'}}>
        <div style={{padding:'14px',background:T.surface,borderRadius:12,border:'1px solid '+T.border,
          marginBottom:18}}>
          <div style={{fontSize:15,fontWeight:600,color:T.fg,fontFamily:T.font,lineHeight:1.3}}>
            {block.title}
          </div>
          <div style={{fontSize:12,color:T.muted,fontFamily:T.font,marginTop:6}}>
            {block.time}–{block.end} · {block.project}
          </div>
        </div>
        <div style={{display:'flex',flexDirection:'column',gap:8}}>
          <button onClick={()=>{const src=block.__sourceDate||TODAY;
            onReschedule(block,'reschedule',src,{initialDate:src,lockDate:true});}} style={{padding:'12px 0',
            borderRadius:10,background:T.accent+'22',border:'1px solid '+T.accent+'55',cursor:'pointer',
            color:T.accent,fontSize:13,fontWeight:600,fontFamily:T.font}}>Изменить время</button>
          <button onClick={()=>{const src=block.__sourceDate||TODAY;
            onReschedule(block,'reschedule',src,{initialDate:src===TODAY?NEXT_DAY:src,lockDate:false});}} style={{padding:'12px 0',
            borderRadius:10,background:T.surface2,border:'1px solid '+T.border,cursor:'pointer',
            color:T.fg,fontSize:13,fontWeight:600,fontFamily:T.font}}>Перенести</button>
          <button onClick={()=>onComplete(block)} style={{padding:'12px 0',borderRadius:10,
            background:T.accentGreen,border:'none',cursor:'pointer',color:T.bg,
            fontSize:13,fontWeight:600,fontFamily:T.font}}>Завершить</button>
        </div>
      </div>
    </Sheet>
  );
}

/* ── Demo controls — outside phone frame, for review only ── */
function DemoControls({appState,setTab}){
  const{setDayStatus,setDayEnergy,setActiveSession,setDayCheckIn}=appState;
  const setHero=(state)=>{
    if(state==='pre'){
      setDayStatus('day_not_started');setDayCheckIn(null);setActiveSession(null);setDayEnergy(0);
    }else if(state==='active'){
      setDayStatus('active');
      setDayCheckIn({energy:2,focus:2,mood:'good',obstacle:''});
      setDayEnergy(2);
      setActiveSession(null);
    }else if(state==='ready'){
      setDayStatus('ready');
      setDayCheckIn({energy:2,focus:2,mood:'good',obstacle:''});
      setActiveSession(null);
    }else if(state==='paused'){
      setDayStatus('active');
      setDayCheckIn({energy:2,focus:2,mood:'good',obstacle:''});
      setActiveSession({kind:'planned',taskId:'t5',blockId:'s5',startedAt:Date.now()-120000,
        accumulatedSec:120,paused:true,accumulatedMs:120000,time:'15:15',end:'16:35'});
    }else if(state==='eod'){
      setDayStatus('end_of_day');
      setDayCheckIn({energy:1,focus:1,mood:'ok',obstacle:'устал'});
      setActiveSession(null);
    }else if(state==='done'){
      setDayStatus('day_completed');
      setDayCheckIn({energy:1,focus:1,mood:'ok',obstacle:''});
      setActiveSession(null);
    }
  };
  const states=[
    {id:'pre',label:'Pre'},
    {id:'ready',label:'Ready'},
    {id:'active',label:'Active'},
    {id:'paused',label:'Paused'},
    {id:'eod',label:'EOD'},
    {id:'done',label:'Done'},
  ];
  return(
    <div style={{display:'flex',flexDirection:'column',gap:12,padding:14,background:T.surface,
      borderRadius:14,border:'1px solid '+T.border,minWidth:180}}>
      <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
        textTransform:'uppercase',fontWeight:600}}>Demo state</div>
      <div style={{display:'flex',flexDirection:'column',gap:6}}>
        {states.map(s=>(
          <button key={s.id} onClick={()=>setHero(s.id)} style={btnDemo()}>{s.label}</button>
        ))}
      </div>
      <div style={{borderTop:'1px solid '+T.border,paddingTop:10,marginTop:4}}>
        <div style={{fontSize:11,color:T.muted2,fontFamily:T.font,letterSpacing:'0.04em',
          textTransform:'uppercase',fontWeight:600,marginBottom:6}}>Tab</div>
        <button onClick={()=>setTab('plan')} style={btnDemo()}>План</button>
      </div>
    </div>
  );
}
function btnDemo(){
  return{padding:'8px 12px',borderRadius:8,border:'1px solid '+T.border,
    background:T.surface2,color:T.muted,fontSize:12,fontWeight:500,fontFamily:T.font,cursor:'pointer'};
}

/* ── Stub screens ── */
function StubScreen({label}){
  return(
    <div style={{height:'100%',display:'flex',alignItems:'center',justifyContent:'center',
      flexDirection:'column',gap:12}}>
      <div style={{fontSize:32,opacity:0.3}}>🚧</div>
      <div style={{fontSize:17,fontWeight:600,color:T.fg,fontFamily:T.font}}>{label}</div>
      <div style={{fontSize:13,color:T.muted2,fontFamily:T.font}}>Скоро</div>
    </div>
  );
}

/* ── Device frame + phone shell ── */
function DeviceFrame({children}){
  return(
    <div style={{width:420,background:'#1a1a1a',borderRadius:48,padding:10,
      boxShadow:'0 30px 60px rgba(0,0,0,0.5), 0 0 0 1px rgba(255,255,255,0.04) inset',
      display:'flex',flexDirection:'column'}}>
      <div style={{width:400,height:895,background:T.bg,borderRadius:38,overflow:'hidden',
        position:'relative',display:'flex',flexDirection:'column'}}>
        {children}
      </div>
    </div>
  );
}
function PhoneShell({children,tab,setTab}){
  return(
    <div style={{height:'100%',display:'flex',flexDirection:'column',position:'relative'}}>
      <StatusBar/>
      <div style={{flex:1,overflow:'hidden',position:'relative'}}>
        {children}
      </div>
      <TabBar tab={tab} setTab={setTab}/>
      <HomeIndicator/>
    </div>
  );
}
function StatusBar(){
  return(
    <div style={{height:44,padding:'0 24px',display:'flex',alignItems:'center',justifyContent:'space-between',
      background:T.bg,flexShrink:0}}>
      <span style={{fontSize:15,fontWeight:600,color:T.fg,fontFamily:T.font}}>9:41</span>
      <div style={{position:'absolute',left:'50%',top:8,transform:'translateX(-50%)',
        width:120,height:28,background:'#000',borderRadius:14}}/>
      <div style={{display:'flex',gap:6,alignItems:'center'}}>
        <svg width="16" height="11" viewBox="0 0 16 11" fill={T.fg}><path d="M1 9h2v1H1zM5 7h2v3H5zM9 5h2v5H9zM13 3h2v7h-2z"/></svg>
        <svg width="14" height="11" viewBox="0 0 14 11" fill="none" stroke={T.fg} strokeWidth="1"><path d="M1 5.5C3.5 2 10.5 2 13 5.5"/><path d="M3 7.5C5 5 9 5 11 7.5"/><circle cx="7" cy="9.5" r="1" fill={T.fg}/></svg>
        <div style={{width:24,height:11,borderRadius:3,border:'1px solid '+T.fg,position:'relative',padding:1}}>
          <div style={{width:'78%',height:'100%',background:T.fg,borderRadius:1}}/>
        </div>
      </div>
    </div>
  );
}
function HomeIndicator(){
  return(
    <div style={{position:'absolute',bottom:6,left:'50%',transform:'translateX(-50%)',
      width:134,height:5,background:T.fg,borderRadius:3,opacity:0.6,pointerEvents:'none'}}/>
  );
}
function TabBar({tab,setTab}){
  const items=[
    {id:'today',label:'Сегодня',icon:iconToday},
    {id:'plan',label:'План',icon:iconPlan},
    {id:'projects',label:'Проекты',icon:iconProjects},
    {id:'time',label:'Время',icon:iconTime},
    {id:'settings',label:'Ещё',icon:iconSettings},
  ];
  return(
    <div style={{height:64,background:T.bg,borderTop:'1px solid '+T.border,display:'flex',
      alignItems:'center',justifyContent:'space-around',flexShrink:0,paddingBottom:8}}>
      {items.map(it=>{
        const active=tab===it.id;
        return(
          <button key={it.id} onClick={()=>setTab(it.id)} style={{flex:1,background:'none',border:'none',
            display:'flex',flexDirection:'column',alignItems:'center',gap:3,padding:'8px 0',cursor:'pointer'}}>
            <div style={{color:active?T.accent:T.muted2,display:'flex'}}>{it.icon(active)}</div>
            <span style={{fontSize:10,fontWeight:active?600:500,
              color:active?T.accent:T.muted2,fontFamily:T.font,letterSpacing:'0.01em'}}>{it.label}</span>
          </button>
        );
      })}
    </div>
  );
}
function iconToday(active){
  return <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke={active?T.accent:T.muted2} strokeWidth="1.6">
    <circle cx="11" cy="11" r="8"/><path d="M11 6v5l3 2"/></svg>;
}
function iconPlan(active){
  return <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke={active?T.accent:T.muted2} strokeWidth="1.6">
    <rect x="3" y="4" width="16" height="15" rx="2"/><line x1="3" y1="9" x2="19" y2="9"/>
    <line x1="7" y1="2" x2="7" y2="6"/><line x1="15" y1="2" x2="15" y2="6"/></svg>;
}
function iconProjects(active){
  return <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke={active?T.accent:T.muted2} strokeWidth="1.6">
    <path d="M3 6l5-3 5 3 5-3v12l-5 3-5-3-5 3z"/><line x1="8" y1="3" x2="8" y2="15"/>
    <line x1="13" y1="6" x2="13" y2="18"/></svg>;
}
function iconTime(active){
  return <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke={active?T.accent:T.muted2} strokeWidth="1.6">
    <circle cx="11" cy="11" r="8"/><path d="M11 5v6l4 2"/></svg>;
}
function iconSettings(active){
  return <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke={active?T.accent:T.muted2} strokeWidth="1.6">
    <circle cx="11" cy="11" r="3"/><path d="M11 2v3M11 17v3M2 11h3M17 11h3M4.5 4.5l2 2M15.5 15.5l2 2M4.5 17.5l2-2M15.5 6.5l2-2"/></svg>;
}

ReactDOM.createRoot(document.getElementById('root')).render(<App/>);
